use std::collections::HashMap;

use crate::addons::fighter_history::FighterHistoryItem;

#[derive(Clone)]
pub struct HistoryItem {
    pub frames: u32,
    pub movement: String,
    pub basic_actions: Vec<String>,
    pub composite_actions: Vec<String>,
    pub charge: HashMap<String, u32>,
    pub all: String,
}

impl HistoryItem {
    pub fn new(movement: impl Into<String>, basic_actions: Vec<String>, composite_actions: Vec<String>, all: impl Into<String>, charge: HashMap<String, u32>) -> Self {
        Self {
            frames: 1,
            movement: movement.into(),
            basic_actions,
            composite_actions,
            all: all.into(),
            charge
        }
    }

    pub fn to_godot(&self) -> FighterHistoryItem {
        FighterHistoryItem::new(self)
    }
}
