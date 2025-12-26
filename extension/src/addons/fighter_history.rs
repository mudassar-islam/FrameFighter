use godot::prelude::*;

use crate::internal::input_history::HistoryItem;

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct FighterHistoryItem {
    #[var]
    frames: u32,

    #[var]
    movement: GString,

    #[var]
    basic_actions: Array<GString>,

    #[var]
    composite_actions: Array<GString>,

    #[var]
    charge: VarDictionary
}

impl FighterHistoryItem {
    pub fn new(history_item: &HistoryItem) -> Self {
        let basic_actions: Array<GString> = Array::from_iter(history_item.basic_actions.iter().map(GString::from));
        let composite_actions: Array<GString> = Array::from_iter(history_item.composite_actions.iter().map(GString::from));
        let mut charge = vdict! {};

        for (key, value) in history_item.charge.iter() {
            charge.set(key.as_str(), *value);
        }

        Self {
            frames: history_item.frames,
            movement: GString::from(&history_item.movement),
            basic_actions,
            composite_actions,
            charge
        }
    }
}

impl Default for FighterHistoryItem {
    fn default() -> Self {
        Self {
            frames: 1,
            movement: GString::from("neutral"),
            basic_actions: array![],
            composite_actions: array![],
            charge: vdict! {}
        }
    }
}
