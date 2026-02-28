use godot::prelude::*;

use crate::internal::input_history::HistoryItem;

#[derive(GodotClass)]
#[class(init, base=Object)]
/// A single input history item. Contains the actions and charge for the **current frame** as-well as how long the current set of actions was pressed for.
pub struct FighterHistoryItem {
    /// Number of frames the current actions were held for.
    #[var]
    frames: u32,

    /// The active movement action.
    #[var]
    movement: GString,

    /// Array of active basic actions.
    #[var]
    basic_actions: Array<GString>,

    /// Array of active basic actions.
    #[var]
    composite_actions: Array<GString>,

    /// Dictionary of chargeable actions and how long they've been pressed for.
    #[var]
    charge: VarDictionary
}

#[godot_api]
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

    /// Used to check if an action is currently pressed.
    #[func]
    pub fn is_pressed(&self, action: GString) -> bool {
        self.movement == action
        ||self.basic_actions.contains(&action)
        ||self.composite_actions.contains(&action)
    }

    /// Used to get the charge amount for a specific action.
    #[func]
    pub fn get_charge_frames(&self, action: GString) -> Variant {
        if let Some(charge) = self.charge.get(action) {
            return charge
        }

        godot_script_error!("Charge Key not found.");
        Variant::from(0)
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
