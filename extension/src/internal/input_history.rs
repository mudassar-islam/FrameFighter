use godot::prelude::*;

use crate::{addons::fighter_history::FighterHistoryItem, internal::{models::frame_input_state::FrameInputState, models::history_item::HistoryItem}};

#[derive(Clone)]
pub struct InputHistory {
    pub size: usize,
    pub max_frames: u32,
    pub entries: Vec<HistoryItem>
}

impl InputHistory {
    pub fn set_max_frames(&mut self, max_frames: u32) {
        self.max_frames = max_frames;
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size.into();
    }

    pub fn add(&mut self, state: &FrameInputState) {
        // Increment the latest entry's frames if input signature is the same;
        if let Some(previous) = self.entries.first_mut() && previous.all == state.all {
            previous.frames = (previous.frames + 1).clamp(0, self.max_frames);
            previous.charge = state.charge.clone();
            return;
        }

        // Otherwise insert a new entry
        self.entries.insert(0, HistoryItem::new(
            state.movement.clone(),
            state.basic_actions.clone(),
            state.composite_actions.clone(),
            &state.all.clone(),
            state.charge.clone()
        ));

        self.entries.truncate(self.size);
    }

    pub fn pressed_actions_for_godot(&self) -> FighterHistoryItem {
        if let Some(current) = self.entries.first() {
            return current.to_godot()
        }

        FighterHistoryItem::default()
    }

    pub fn to_godot(&mut self) -> Array<Gd<FighterHistoryItem>> {
        Array::from_iter(self.entries.iter().map(|e| Gd::from_object(e.to_godot())))
    }
}

impl Default for InputHistory {
    fn default() -> Self {
        Self {
            size: 20,
            max_frames: 99,
            entries: vec![]
        }
    }
}
