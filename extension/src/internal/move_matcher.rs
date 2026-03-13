use godot::prelude::*;
use indexmap::IndexMap;

use crate::{
    addons::{fighter_history::FighterMatchedMove, frame_fighter::FrameFighter},
    internal::models::{frame_input_state::FrameInputState, history_item::HistoryItem, move_def::{MatchedMove, Move}, sequence_step::SequenceStep}
};

#[derive(PartialEq)]
enum StepResult {
    Invalid,
    Perfect,
    TwoStep,
    ThreeStep
}

pub struct MoveMatcher {
    matched_moves: Vec<MatchedMove>,
    buffer: Vec<HistoryItem>,
    buffer_stripped: Vec<HistoryItem>,
    size: usize,
    max_frames: u32,
    moves: IndexMap<String, Move>,
}

impl MoveMatcher {
    pub fn add_move(
        &mut self,
        name: impl Into<String>,
        sequence: Vec<SequenceStep>,
        require_neutrals: bool,
        require_charge: bool,
        charge_frames: u32,
        priority: i32
    ) {
        self.moves.insert(
            name.into(),
            Move::new(sequence, require_neutrals, require_charge, charge_frames, priority),
        );
    }

    // Sort moves with longest sequence first.
    pub fn sort_moves(&mut self) {
        self.moves
            .sort_by(|_, m1, _, m2| m2.reversed_sequence.len().cmp(&m1.reversed_sequence.len()));
    }

    pub fn matched_moves_for_godot(&mut self) -> Array<Gd<FighterMatchedMove>> {
        Array::from_iter(self.matched_moves.iter().map(|m| {
            Gd::from_object(FighterMatchedMove {
                name: GString::from(&m.name),
                total_frames: m.total_frames,
                perfect_input: m.perfect_input,
                priority: m.priority
            })
        }))
    }

    pub fn process_frame(&mut self) {
        self.matched_moves.clear();

        for (name, move_def) in &self.moves {
            let (success, perfect_input, total_frames) = self.match_move(move_def);
            if success {
                self.matched_moves.push(MatchedMove {
                    name: name.to_string(),
                    priority: move_def.priority,
                    perfect_input,
                    total_frames,
                });
            }
        }

        if self.matched_moves.len() > 0 {
            self.matched_moves.sort_by(|m1, m2| m1.priority.cmp(&m2.priority));
            self.clear_buffers();
        }
    }

    fn clear_buffers(&mut self) {
        self.buffer.clear();
        self.buffer_stripped.clear();
    }

    fn match_move(&self, move_def: &Move) -> (bool, bool, u32) {
        let seq = &move_def.reversed_sequence;
        let buf = match move_def.require_neutrals {
            true => &self.buffer,
            false => &self.buffer_stripped
        };

        if seq.is_empty() || buf.is_empty() || buf.len() < seq.len() {
            return (false, false, 0);
        }

        let mut buf_idx = 0;
        let mut matches = 0;
        let mut perfect_input = true;
        let mut total_frames = 0;

        'attempt: for (step_idx, step) in seq.iter().enumerate() {
            let mut step_frames = 0;
            let mut step_matched = false;

            while buf_idx < buf.len() {
                let current = &buf[buf_idx];

                // Only count frames if not the first input. Because the first input can be held for as long as possible.
                if step_idx != seq.len() - 1 {
                    step_frames += current.frames;
                }

                let result =
                    self.satisfies_step(move_def, step, current, step_idx, buf_idx);

                if result == StepResult::Invalid {
                    break 'attempt;
                }

                buf_idx += 1;

                match result {
                    StepResult::TwoStep => {
                        self.print_buffers();

                        step_frames += &buf[buf_idx + 1].frames;
                        perfect_input = false;
                        buf_idx += 1;
                    },
                    StepResult::ThreeStep => {
                        self.print_buffers();

                        step_frames += &buf[buf_idx + 1].frames;
                        step_frames += &buf[buf_idx + 2].frames;
                        perfect_input = false;
                        buf_idx += 2;
                    },
                    _ => ()
                };

                step_matched = true;

                break;
            }

            total_frames += step_frames;

            if step_matched && step_frames <= step.input_window {
                matches += 1;
            }

            if matches == seq.len() {
                return (true, perfect_input, total_frames);
            }
        }

        return (false, false, 0);
    }

    fn satisfies_step(
        &self,
        move_def: &Move,
        step: &SequenceStep,
        current: &HistoryItem,
        step_idx: usize,
        buf_idx: usize,
    ) -> StepResult {
        let is_first_step = step_idx == move_def.reversed_sequence.len() - 1;
        let is_last_step = step_idx == 0;

        let mut result = StepResult::Perfect;

        if move_def.require_charge {
            let (movement, actions) = (
                MoveMatcher::charge_movement_match(current, step),
                MoveMatcher::actions_match(current, step)
            );

            if is_first_step && let Some(charge) = current.charge.get(&step.movement) && *charge < move_def.charge_frames {
                result = StepResult::Invalid;
            }

            if !movement || !actions {
                result = StepResult::Invalid;
            }

            return result;
        }

        let (movement, actions) = (
            MoveMatcher::strict_movement_match(current, step),
            MoveMatcher::actions_match(current, step),
        );

        if !movement || !actions {
            result = StepResult::Invalid;
        }

        let prev_a = self.buffer.get(buf_idx + 1);
        let prev_b = self.buffer.get(buf_idx + 2);

        if let Some(prev_a) = prev_a
            && is_last_step
            && step.modifiers.contains(&FrameFighter::LENIENT_ENDER)
            && actions
        {
            if MoveMatcher::lenient_movement_match(prev_a, step)
                && (MoveMatcher::neutral_movement(current) || movement)
            {
                result = StepResult::TwoStep;
            } else if let Some(prev_b) = prev_b
                && MoveMatcher::lenient_movement_match(prev_b, step)
                && (MoveMatcher::neutral_movement(prev_a) || MoveMatcher::lenient_movement_match(prev_a, step))
                && (MoveMatcher::neutral_movement(current))
            {
                result = StepResult::ThreeStep;
            }
        }

        /* if(input.movement == "down_forward" && input.basic_actions.contains(&"lp".to_string())) {
            godot_print!("But it's not");
        } */

        // If it's a charge move, we match movement differently.
        // Diagonals should still count as valid as-long as they have enough charge for the starting direction.
        // Back -> Forward + Punch should still be matched with an input of Down Back -> Down Forward + Punch.
        /* if move_def.require_charge {

            // Invalidate move attempt immediately if not charged for required duration.
            if is_charge_step && let Some(input_frames) = input.charge.get(&step.movement) && *input_frames < charge_frames {
                return StepResult::INVALID;
            }

            // If charged for required duration, loosely check the movement.
            if (is_charge_step || is_last_step) && !MoveMatcher::cardinal_match(input, step) {
                return StepResult::INVALID;
            }

            return StepResult::PERFECT;

        } */

        result
    }

    fn print_buffers(&self) {
        let mut buf: Vec<String> = vec![];
        let mut buf_stripped: Vec<String> = vec![];

        for item in self.buffer.iter() {
            let mut st = "{".to_string();

            st.push_str(&item.movement.to_string());
            st.push_str(".");
            st.push_str(&item.basic_actions.join(","));
            st.push_str("}");

            buf.push(st);
        }

        for item in self.buffer_stripped.iter() {
            let mut st = "{".to_string();

            st.push_str(&item.movement.to_string());
            st.push_str(".");
            st.push_str(&item.basic_actions.join(","));
            st.push_str("}");

            buf_stripped.push(st);
        }

        godot_print!("Standard: [ {} ]", buf.join(" | "));
        godot_print!("Stripped: [ {} ]", buf_stripped.join(" | "));
    }

    fn strict_movement_match(input: &HistoryItem, step: &SequenceStep) -> bool {
        step.movement.is_empty() || step.movement == input.movement
    }

    // For charge moves, we only wanna check if the cardinal direction exists.
    // ⇐⇒(B) : Should still be triggered with ⇙⇘(B)
    fn charge_movement_match(input: &HistoryItem, step: &SequenceStep) -> bool {
        step.movement.is_empty() || input.movement.contains(&step.movement)
    }

    fn lenient_movement_match(input: &HistoryItem, step: &SequenceStep) -> bool {
        /* if !FrameFighter::is_cardinal(&step.movement) {

        } */

        step.movement.is_empty() || step.movement == input.movement
    }

    fn neutral_movement(input: &HistoryItem) -> bool {
        input.movement == "neutral"
    }

    /* fn cardinal_match(input: &HistoryItem, step: &SequenceStep) -> bool {
        step.movement.is_empty()
        || input.movement.contains(&step.movement)
        || !input.movement.contains("_")
    } */

    fn actions_match(input: &HistoryItem, step: &SequenceStep) -> bool {
        /* godot_print!("Step: {} - {}", step.movement, step.actions.join("."));
        godot_print!(
            "Input: {} - {}\n",
            input.movement,
            input.basic_actions.join(".")
        ); */

        match step.actions.len() {
            0 => input.basic_actions.len() + input.composite_actions.len() == 0,
            _ => step.actions.iter().all(|required_action| {
                input.basic_actions.contains(required_action)
                    || input.composite_actions.contains(required_action)
            }),
        }
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    pub fn set_max_frames(&mut self, max_frames: u32) {
        self.max_frames = max_frames;
    }

    // Shamelessly repeated from InputHistory
    pub fn add_buffer_entry(&mut self, state: &FrameInputState) {
        // Increment the latest entry's frames if input signature is the same;
        if let Some(previous) = self.buffer.first_mut()
            && previous.all == state.all
        {
            previous.frames = (previous.frames + 1).clamp(0, self.max_frames);
            previous.charge = state.charge.clone();

            // Add the current entry's frames to the last non-empty input in-case it's the same signature.
            if let Some(previous_stripped) = self.buffer_stripped.first_mut() {
                previous_stripped.frames = (previous_stripped.frames + 1).clamp(0, self.max_frames);
                previous_stripped.charge = state.charge.clone();
            }

            return;
        }

        // Otherwise insert a new entry
        self.buffer.insert(
            0,
            HistoryItem::new(
                state.movement.clone(),
                state.basic_actions.clone(),
                state.composite_actions.clone(),
                state.all.clone(),
                state.charge.clone(),
            ),
        );

        // If it's not an empty input, we add it to the stripped buffer.
        if !(state.movement == "neutral" && state.basic_actions.len() + state.composite_actions.len() == 0) && let Some(prev) = self.buffer.first() {
            self.buffer_stripped.insert(0, prev.clone());
        }

        self.buffer.truncate(self.size);
        self.buffer_stripped.truncate(self.size);
    }
}

impl Default for MoveMatcher {
    fn default() -> Self {
        Self {
            matched_moves: vec![],
            size: 20,
            max_frames: 999,
            moves: IndexMap::new(),
            buffer: vec![],
            buffer_stripped: vec![],
        }
    }
}
