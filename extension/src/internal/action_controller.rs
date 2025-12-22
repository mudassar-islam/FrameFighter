use std::collections::HashMap;
use godot::{classes::Input, global::{godot_error}, obj::Singleton};
use indexmap::IndexMap;

use crate::addons::frame_fighter::FrameFighter;

pub enum ActionType {
    Basic {
        input_action: String,
        is_dependency: bool
    },

    Composite {
        dependencies: Vec<String>,
        require_all: bool
    }
}

pub struct Action {
    pressed: bool,
    charge_type: i32,
    action_type: ActionType
}

impl Action {
    pub fn basic(input_action: impl Into<String>, charge_type: i32) -> Self {
        Self {
            pressed: false,
            charge_type,
            action_type: ActionType::Basic {
                input_action: input_action.into(),
                is_dependency: false
            }
        }
    }

    pub fn composite(dependencies: Vec<impl Into<String>>, charge_type: i32, require_all: bool) -> Self {
        Self {
            pressed: false,
            charge_type,
            action_type: ActionType::Composite {
                dependencies: dependencies.into_iter().map(|d| d.into()).collect(),
                require_all
            }
        }
    }
}

#[derive(Default)]
pub struct FrameInputState {
    pub movement: String,
    pub basic_actions: Vec<String>,
    pub composite_actions: Vec<String>,
    pub charge: HashMap<String, i32>,
}

impl FrameInputState {
    pub fn new(movement: impl Into<String>, basic_actions: Vec<impl Into<String>>, composite_actions: Vec<impl Into<String>>, charge: HashMap<String, i32>) -> Self {
        Self {
            movement: movement.into(),
            basic_actions: basic_actions.into_iter().map(|a| a.into()).collect(),
            composite_actions: composite_actions.into_iter().map(|a| a.into()).collect(),
            charge
        }
    }
}

pub struct ActionController {
    side: i32,
    actions: IndexMap<String, Action>,
    charge: HashMap<String, i32>,
    opposites: HashMap<String, (String, String, String)>,
    dependency_input: HashMap<String, bool>,
    can_charge: bool
}

impl Default for ActionController {
    fn default() -> Self {
        Self {
            side: FrameFighter::PLAYER_ONE,
            can_charge: false,
            actions: IndexMap::new(),
            charge: HashMap::new(),
            opposites: HashMap::new(),
            dependency_input: HashMap::new()
        }
    }
}

impl ActionController {
    pub fn bind_directions(&mut self, up: impl Into<String>, down: impl Into<String>, forward: impl Into<String>, back: impl Into<String>, charge_type: i32) {
        let (up, down, forward, back) = (up.into(), down.into(), forward.into(), back.into());

        // Basic actions for movement. Can all be charged.
        self.add("up", &up, charge_type);
        self.add("down", &down, charge_type);
        self.add("forward", &forward, charge_type);
        self.add("back", &back, charge_type);

        // Composite actions for diagonal movement. Cannot be charged and require all dependencies to be pressed
        self.add_composite("up_forward", vec![ "up", "forward" ], FrameFighter::CHARGE_NONE, true);
        self.add_composite("down_forward", vec![ "down", "forward" ], FrameFighter::CHARGE_NONE, true);
        self.add_composite("up_back", vec![ "up", "back" ], FrameFighter::CHARGE_NONE, true);
        self.add_composite("down_back", vec![ "down", "back" ], FrameFighter::CHARGE_NONE, true);

        // Charge keys only for directions.
        self.create_charge_key("up");
        self.create_charge_key("down");
        self.create_charge_key("back");
        self.create_charge_key("forward");

        // A hashmap for the opposite action and opposite input action for every direction.
        // Required for SOCD checks.
        self.build_opposite_map("up", "down", &down, &down);
        self.build_opposite_map("down", "up", &up, &up);
        self.build_opposite_map("back", "forward", &forward, &forward);
        self.build_opposite_map("forward", "back", &back, &back);
    }

    pub fn add(&mut self, name: impl Into<String>, input_action: impl Into<String>, charge_type: i32) {
        self.actions.insert(
            name.into(),
            Action::basic(
                input_action.into(),
                charge_type
            )
        );
    }

    pub fn add_composite(&mut self, name: impl Into<String>, dependencies: Vec<impl Into<String>>, charge_type: i32, require_all: bool) {
        self.actions.insert(
            name.into(),
            Action::composite(
                dependencies.into_iter().map(|s| s.into()).collect(),
                charge_type,
                require_all
            )
        );
    }

    pub fn create_charge_key(&mut self, name: impl Into<String>) {
        self.charge.insert(name.into(), 0);
    }

    pub fn build_opposite_map(&mut self, name: impl Into<String>, opposite: impl Into<String>, current_input: impl Into<String>, original_input: impl Into<String>) {
        self.opposites.insert(name.into(), (opposite.into(), current_input.into(), original_input.into()));
    }

    pub fn build_dependency_map(&mut self) {
        self.dependency_input.clear();

        for action in self.actions.values() {
            if let ActionType::Composite { dependencies, .. } = &action.action_type {
                for dep in dependencies {
                    self.dependency_input.insert(dep.clone(), false);
                }
            }
        }

        for key in self.dependency_input.keys() {
            if let Some(action) = self.actions.get_mut(key) && let ActionType::Basic { is_dependency, .. } = &mut action.action_type {
                *is_dependency = true;
            }
        }
    }

    pub fn should_charge(&mut self, can_charge: bool) {
        self.can_charge = can_charge;
    }

    // I don't like this method.
    // Will possibly revamp how opposites are handled to improve this.
    pub fn handle_side(&mut self, side: i32) {
        if side == self.side {
            return;
        }

        self.side = side;

        let opposites = self.opposites.get_disjoint_mut(["forward", "back"]);
        let actions = self.actions.get_disjoint_mut(["forward", "back"]);

        if let [Some(forward), Some(back)] = actions
            && let [Some(forward_opp), Some(back_opp)] = opposites
            && let ActionType::Basic { input_action: forward_input, .. } = &mut forward.action_type
            && let ActionType::Basic { input_action: back_input, .. } = &mut back.action_type {

            let (_, forward_current, forward_original) = back_opp;
            let (_, back_current, back_original) = forward_opp;

            if self.side == FrameFighter::PLAYER_ONE {
                *forward_current = forward_original.clone();
                *back_current = back_original.clone();
            } else {
                *forward_current = back_original.clone();
                *back_current = forward_original.clone();
            }

            *forward_input = forward_current.clone();
            *back_input = back_current.clone();
        }
    }

    pub fn process_current_frame(&mut self) {
        let input = Input::singleton();

        for (name, action) in self.actions.iter_mut() {
            if let ActionType::Composite { dependencies, require_all } = &action.action_type {
                let mut dependency_state = dependencies.iter().map(|dependency|
                    self.dependency_input.get(dependency) == Some(&true)
                );

                action.pressed =
                    if *require_all {
                        dependency_state.all(|pressed| pressed == true)
                    } else {
                        dependency_state.filter(|&pressed| pressed == true).count() > 1
                    };

            } else if let ActionType::Basic { input_action, is_dependency } = &action.action_type {
                action.pressed = input.is_action_pressed(input_action);

                if action.pressed && FrameFighter::is_movement(name) && let Some((_, opposite_input, _)) = self.opposites.get(name) {
                    if input.is_action_pressed(opposite_input) {
                        action.pressed = false;
                    }
                }

                if *is_dependency && let Some(dependency_input) = self.dependency_input.get_mut(name) {
                    *dependency_input = action.pressed;
                }
            }

            // "charge" a move when the action is pressed and "can_charge"
            // "can_charge" can be modified by the user to disallow charging mid-air for example.
            if action.charge_type != FrameFighter::CHARGE_NONE {
                let charge = self.charge.get_mut(name);

                match charge {
                    Some(frames) => {
                        if action.pressed && self.can_charge {
                            *frames = (*frames + 1).clamp(0, 9999);
                        } else if action.charge_type == FrameFighter::CHARGE_TICK {
                            *frames = (*frames - 1).clamp(0, 9999);
                        } else {
                            *frames = 0;
                        }
                    },
                    None => godot_error!("Charge Key for {} not found.", name)
                };
            }
        }
    }

    pub fn frame_input_state(&mut self) -> FrameInputState {
        let mut movement = "neutral";
        let mut basic: Vec<&str> = Vec::new();
        let mut composite: Vec<&str> = Vec::new();

        for (name, action) in &self.actions {
            if action.pressed {
                if FrameFighter::is_movement(name.as_ref()) {
                    movement = &name;
                } else if let ActionType::Composite { .. } = action.action_type {
                    composite.push(&name);
                } else {
                    basic.push(&name);
                }
            }
        }

        FrameInputState::new(movement, basic, composite, self.charge.clone())
    }
}
