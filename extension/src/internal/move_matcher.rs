use std::{collections::HashMap, hash::Hash};

use godot::prelude::*;
use indexmap::IndexMap;

use crate::{
    addons::{fighter_history::FighterMatchedMove, frame_fighter::FrameFighter},
    internal::action_controller::FrameInputState,
};

#[derive(PartialEq)]
enum StepResult {
    INVALID,
    PERFECT,
    TWO_STEP,
    THREE_STEP,
    MISS,
}

pub struct MoveMatcher {
    matched_moves: Vec<MatchedMove>,
    buffer: Vec<BufferItem>,
    size: usize,
    max_frames: u32,
    moves: IndexMap<String, Move>,
}

impl MoveMatcher {
    pub fn add_move(
        &mut self,
        name: impl Into<String>,
        sequence: Vec<SequenceStep>,
        require_charge: bool,
        charge_frames: u32,
    ) {
        self.moves.insert(
            name.into(),
            Move::new(sequence, require_charge, charge_frames),
        );
    }

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
            })
        }))
    }

    pub fn process_frame(&mut self) {
        self.matched_moves = vec![];

        for (name, move_def) in &self.moves {
            let (success, perfect_input, total_frames) = self.match_move(move_def);
            if success {
                self.matched_moves.push(MatchedMove {
                    name: name.to_string(),
                    perfect_input,
                    total_frames,
                });
            }
        }

        if self.matched_moves.len() > 0 {
            self.buffer.clear();
        }
    }

    fn match_move(&self, move_def: &Move) -> (bool, bool, u32) {
        let seq = &move_def.reversed_sequence;
        let buf = &self.buffer;

        if seq.is_empty() || buf.is_empty() {
            return (false, false, 0);
        }

        let mut buf_idx = 0;
        let mut found = 0;
        let mut total_frames = 0;
        let mut perfect_input = true;

        'attempt: for (step_idx, step) in seq.iter().enumerate() {
            let is_last_step = step_idx == 0;

            while buf_idx < buf.len() {
                let input = &buf[buf_idx];
                total_frames += input.frames;

                let result =
                    self.satisfies_step(move_def, step, input, false, 0, is_last_step, buf_idx);

                if result == StepResult::INVALID {
                    break 'attempt;
                }

                found += 1;
                buf_idx += 1;

                match result {
                    StepResult::TWO_STEP => {
                        buf_idx += 1;
                        total_frames += &buf[buf_idx + 1].frames;
                        perfect_input = false;
                    },
                    StepResult::THREE_STEP => {
                        buf_idx += 2;
                        total_frames += &buf[buf_idx + 1].frames;
                        total_frames += &buf[buf_idx + 2].frames;
                        perfect_input = false;
                    },
                    _ => ()
                };

                break;
            }

            if found == seq.len() {
                return (true, perfect_input, total_frames);
            }
        }

        return (false, false, 0);
    }

    /* fn match_move(&self, move_def: &Move) -> (bool, bool, u32) {
        let seq = &move_def.reversed_sequence;
        let buffer = &self.buffer;

        if seq.is_empty() || buffer.is_empty() {
            return (false, false, 0);
        }

        let mut buf_idx = 0;
        let mut total_frames: u32 = 0;
        let mut perfect_input = true;

        for (step_idx, step) in seq.iter().enumerate() {
            // Whether the current step requires charge, or whether it is the last step.
            // The order of the sequence is reversed so 0 = last, len() - 1 = first
            let is_charge_step = move_def.require_charge && step_idx == seq.len() - 1;
            let is_last_step = step_idx == 0;

            let mut found = false;
            let mut window_frames: u32 = 0;
            let scan_start = buf_idx;

            /* if !FrameFighter::is_cardinal(&step.movement) && step.modifiers.contains(&FrameFighter::IGNORE_DIAGONAL) && step.actions.len() == 0 {
                continue;
            } */

            while buf_idx < buffer.len() {
                let input = &buffer[buf_idx];

                let result = self.satisfies_step(move_def, step, input, is_charge_step, move_def.charge_frames, is_last_step, buf_idx);

                if result == StepResult::INVALID {
                    break;
                }

                if result == StepResult::MISS {
                    // This entry didn't match. Consume its frames against the window.
                    window_frames += input.frames;

                    // The first step has no "previous anchor" so we treat its input_window
                    // as a recency gate: how recently must the final input have occurred?
                    if window_frames >= step.input_window && !is_charge_step {
                        break; // Window expired, move attempt fails
                    }

                    buf_idx += 1;
                }

                if result == StepResult::PERFECT || result == StepResult::TWO_STEP {
                    if buf_idx > scan_start {
                        // Had to skip buffer entries to find a match — not a perfect input
                        perfect_input = false;
                    }

                    total_frames = (total_frames + (window_frames + input.frames)).clamp(1, self.max_frames);
                    found = true;

                    buf_idx += 1;
                    if result == StepResult::TWO_STEP {
                        buf_idx += 1;
                    }

                    break;
                }
            }

            if !found {
                return (false, false, 0);
            }
        }

        (true, perfect_input, total_frames)
    } */

    fn satisfies_step(
        &self,
        move_def: &Move,
        step: &SequenceStep,
        current: &BufferItem,
        is_charge_step: bool,
        charge_frames: u32,
        is_last_step: bool,
        buf_idx: usize,
    ) -> StepResult {
        let mut result = StepResult::PERFECT;

        let (movement, actions) = (
            MoveMatcher::strict_movement_match(current, step),
            MoveMatcher::actions_match(current, step),
        );

        if !movement || !actions {
            result = StepResult::INVALID;
        }

        let previous_a = self.buffer.get(buf_idx + 1);
        let previous_b = self.buffer.get(buf_idx + 2);

        if let Some(previous_a) = previous_a
            && is_last_step
            && step.modifiers.contains(&FrameFighter::LENIENT_ENDER)
            && actions
        {
            if MoveMatcher::lenient_movement_match(previous_a, step)
                && (MoveMatcher::neutral_movement(current) || movement)
            {
                result = StepResult::TWO_STEP;
            } else if let Some(previous_b) = previous_b
                && MoveMatcher::lenient_movement_match(previous_b, step)
                && (MoveMatcher::neutral_movement(previous_a) || MoveMatcher::lenient_movement_match(previous_a, step))
                && (MoveMatcher::neutral_movement(current))
            {
                result = StepResult::THREE_STEP;
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

    fn strict_movement_match(input: &BufferItem, step: &SequenceStep) -> bool {
        step.movement.is_empty() || step.movement == input.movement
    }

    fn lenient_movement_match(input: &BufferItem, step: &SequenceStep) -> bool {
        /* if !FrameFighter::is_cardinal(&step.movement) {

        } */

        step.movement.is_empty() || step.movement == input.movement
    }

    fn neutral_movement(input: &BufferItem) -> bool {
        input.movement == "neutral"
    }

    /* fn cardinal_match(input: &BufferItem, step: &SequenceStep) -> bool {
        step.movement.is_empty()
        || input.movement.contains(&step.movement)
        || !input.movement.contains("_")
    } */

    fn actions_match(input: &BufferItem, step: &SequenceStep) -> bool {
        godot_print!("Step: {} - {}", step.movement, step.actions.join("."));
        godot_print!(
            "Input: {} - {}\n",
            input.movement,
            input.basic_actions.join(".")
        );

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
            return;
        }

        // Otherwise insert a new entry
        self.buffer.insert(
            0,
            BufferItem::new(
                state.movement.clone(),
                state.basic_actions.clone(),
                state.composite_actions.clone(),
                state.charge.clone(),
                state.all.clone(),
            ),
        );

        self.buffer.truncate(self.size);
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
        }
    }
}
#[derive(Clone)]
pub struct BufferItem {
    pub frames: u32,
    pub movement: String,
    pub basic_actions: Vec<String>,
    pub composite_actions: Vec<String>,
    pub charge: HashMap<String, u32>,
    pub all: String,
}

impl BufferItem {
    pub fn new(
        movement: impl Into<String>,
        basic_actions: Vec<String>,
        composite_actions: Vec<String>,
        charge: HashMap<String, u32>,
        all: impl Into<String>,
    ) -> Self {
        Self {
            frames: 1,
            movement: movement.into(),
            basic_actions,
            composite_actions,
            charge,
            all: all.into(),
        }
    }
}

#[derive(Clone)]
pub struct SequenceStep {
    pub movement: String,
    pub actions: Vec<String>,
    pub input_window: u32,
    pub modifiers: Vec<u32>,
}

impl SequenceStep {
    pub fn new(
        movement: impl Into<String>,
        actions: Vec<impl Into<String>>,
        input_window: u32,
        modifiers: Vec<u32>,
    ) -> Self {
        Self {
            movement: movement.into(),
            actions: actions.into_iter().map(|a| a.into()).collect(),
            input_window,
            modifiers,
        }
    }
}

pub struct Move {
    pub reversed_sequence: Vec<SequenceStep>,
    pub require_charge: bool,
    pub charge_frames: u32,
}

impl Move {
    pub fn new(sequence: Vec<SequenceStep>, require_charge: bool, charge_frames: u32) -> Self {
        let mut reversed_sequence = sequence.clone();
        reversed_sequence.reverse();

        Self {
            reversed_sequence,
            require_charge,
            charge_frames,
        }
    }
}

pub struct MatchedMove {
    pub name: String,
    pub total_frames: u32,
    pub perfect_input: bool,
}
