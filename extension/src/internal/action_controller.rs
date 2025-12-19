use std::{collections::HashMap};
use godot::{classes::Input, global::{godot_error}, obj::Singleton};

use crate::addons::frame_fighter::FrameFighter;

#[derive(Clone)]
pub struct Action {
    name: String,
    composite: bool,
    pressed: bool,
    charge_type: i32,

    input_action: String,
    dependencies: Vec<String>,
    is_dependency: bool,
    require_all: bool,
}

impl Action {
    pub fn basic(name: String, input_action: String, charge_type: i32) -> Self {
        Self {
            name,
            composite: false,
            pressed: false,
            charge_type,
            is_dependency: false,

            input_action,
            dependencies: vec![],
            require_all: false
        }
    }

    pub fn composite(name: String, dependencies: Vec<String>, charge_type: i32, require_all: bool) -> Self {
        Self {
            name,
            composite: true,
            pressed: false,
            charge_type,
            is_dependency: false,

            input_action: String::from("none"),
            dependencies,
            require_all
        }
    }

    pub fn is_movement(&self) -> bool {
        [ "up", "down", "back", "forward", "up_forward", "up_back", "down_forward", "down_back" ].contains(&self.name.as_str())
    }
}

pub struct ActionController {
    back_input_action: String,
    forward_input_action: String,

    actions: HashMap<String, Action>,
    charge: HashMap<String, i32>,
    opposites: HashMap<String, String>,
    dependency_input: HashMap<String, bool>,
    actions_order: Vec<String>,

    can_charge: bool,

    pressed_actions_movement: String,
    pressed_actions_basic: Vec<String>,
    pressed_actions_composite: Vec<String>,
}

impl ActionController {
    pub fn bind_directions(&mut self, up: String, down: String, forward: String, backward: String, charge_type: i32) {
        self.add("up", &up, charge_type);
        self.add("down", &down, charge_type);
        self.add("forward", &forward, charge_type);
        self.add("back", &backward, charge_type);

        self.add_composite("up_forward", vec![ "up", "forward" ], FrameFighter::CHARGE_NONE, true);
        self.add_composite("down_forward", vec![ "down", "forward" ], FrameFighter::CHARGE_NONE, true);
        self.add_composite("up_back", vec![ "up", "back" ], FrameFighter::CHARGE_NONE, true);
        self.add_composite("down_back", vec![ "down", "back" ], FrameFighter::CHARGE_NONE, true);

        self.create_charge_key("up", 0);
        self.create_charge_key("down", 0);
        self.create_charge_key("back", 0);
        self.create_charge_key("forward", 0);

        self.opposites.insert("up".to_string(), "down".to_string());
        self.opposites.insert("down".to_string(), "up".to_string());
        self.opposites.insert("back".to_string(), "forward".to_string());
        self.opposites.insert("forward".to_string(), "back".to_string());

        self.back_input_action = backward.clone();
        self.forward_input_action = forward.clone();
    }

    pub fn add(&mut self, name: &str, input_action: &str, charge_type: i32) {
        self.actions.insert(
            name.to_string(),
            Action::basic(
                name.to_string(),
                input_action.to_string(),
                charge_type
            )
        );

        if charge_type != FrameFighter::CHARGE_NONE {
            self.create_charge_key(name, 0);
        }

        self.actions_order.push(name.to_string());
    }

    pub fn add_composite(&mut self, name: &str, dependencies: Vec<&str>, charge_type: i32, require_all: bool) {
        for dependency in dependencies.iter() {
            self.dependency_input.insert(dependency.to_string(), false);

            if let Some(action) = self.actions.get_mut(&dependency.to_string()) {
                action.is_dependency = true;
            }
        }

        self.actions.insert(
            name.to_string(),
            Action::composite(
                name.to_string(),
                dependencies.iter().map(|s| s.to_string()).collect(),
                charge_type,
                require_all
            )
        );

        if charge_type != FrameFighter::CHARGE_NONE {
            self.create_charge_key(name, 0);
        }

        self.actions_order.push(name.to_string());
    }

    pub fn create_charge_key(&mut self, name: &str, frames: i32) {
        self.charge.insert(name.to_string(), frames);
    }

    pub fn should_charge(&mut self, can_charge: bool) {
        self.can_charge = can_charge;
    }

    pub fn handle_side(&mut self, side: i32) {
        let forward_action = self.actions.get_mut("forward");
        match forward_action {
            Some(forward_action) => {
                if side == FrameFighter::PLAYER_ONE {
                    forward_action.input_action = self.forward_input_action.clone();
                } else {
                    forward_action.input_action = self.back_input_action.clone();
                }
            },
            _ => godot_error!("Forward action not set.")
        }
        let back_action = self.actions.get_mut("back");
        match back_action {
            Some(back_action) => {
                if side == FrameFighter::PLAYER_ONE {
                    back_action.input_action = self.back_input_action.clone();
                } else {
                    back_action.input_action = self.forward_input_action.clone();
                }
            },
            _ => godot_error!("Backward action not set.")
        }
    }

    pub fn process_current_frame(&mut self) {
        let input = Input::singleton();

        // Used to check if dependencies
        let cloned_actions = self.actions.clone();
        let mut released_actions = vec![];

        for name in self.actions_order.iter() {
            if let Some(action) = self.actions.get_mut(name) {
                match action.composite {
                    true => {
                        let mut dependency_state = action.dependencies.iter()
                            .map(|dependency| cloned_actions.get(dependency))
                            .map(|sub_action|
                                match sub_action {
                                    Some(sub_action) => {
                                        if let Some(&is_true) = self.dependency_input.get(&sub_action.name) && is_true {
                                            true
                                        } else {
                                            false
                                        }
                                    },
                                    None => false
                                }
                            );

                        action.pressed =
                            if action.require_all {
                                dependency_state.all(|pressed| pressed == true)
                            } else {
                                dependency_state.filter(|&pressed| pressed == true).count() > 1
                            };

                        // "release" the dependency actions in case of a diagonal because we want a single direction.
                        if action.pressed && action.is_movement() {
                            for dependency in action.dependencies.iter() {
                                released_actions.push(dependency.clone());
                            }
                        }
                    },

                    false => {
                        action.pressed = input.is_action_pressed(&action.input_action);

                        if action.pressed && action.is_movement() {
                            // SOCD Check
                            let opposite = self.opposites.get(&action.name);

                            match opposite {
                                Some(opposite) => {
                                    if let Some(opposite_action) = cloned_actions.get(opposite) && input.is_action_pressed(&opposite_action.input_action) {
                                        action.pressed = false;
                                        released_actions.push(opposite_action.name.clone());
                                    }
                                },

                                _ => godot_error!("Opposite direction for cardinal {} not found.", &action.name)
                            }
                        }

                        if action.is_dependency {
                            self.dependency_input.insert(action.name.clone(), action.pressed);
                        }
                    }
                }

                // "charge" a move when the action is pressed and "can_charge"
                // "can_charge" can be modified by the user to disallow charging mid-air for example.
                if action.charge_type != FrameFighter::CHARGE_NONE {
                    let charge = self.charge.get_mut(&action.name);

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
                        None => panic!("Charge Key for {} not found.", &action.name)
                    };
                }
            }
        }

        // Apply released actions
        for name in released_actions.iter() {
            if let Some(action) = self.actions.get_mut(name.as_str()) {
                action.pressed = false;
            }
        }
    }

    pub fn frame_input_state(&mut self) -> FrameInputState {
        self.pressed_actions_movement = "neutral".to_string();
        self.pressed_actions_basic.clear();
        self.pressed_actions_composite.clear();

        for (name, action) in self.actions.iter() {
            if action.pressed {
                if action.is_movement() {
                    self.pressed_actions_movement = name.to_string();
                } else if action.composite {
                    self.pressed_actions_composite.push(name.to_string());
                } else {
                    self.pressed_actions_basic.push(name.to_string());
                }
            }
        }

        FrameInputState {
            movement: self.pressed_actions_movement.clone(),
            basic_actions: self.pressed_actions_basic.clone(),
            composite_actions: self.pressed_actions_composite.clone(),
            charge: self.charge.clone()
        }
    }
}

impl Default for ActionController {
    fn default() -> Self {
        Self {
            back_input_action: "m_back".to_string(),
            forward_input_action: "m_forward".to_string(),
            can_charge: false,

            actions: HashMap::new(),
            charge: HashMap::new(),
            opposites: HashMap::new(),
            dependency_input: HashMap::new(),
            actions_order: vec![],

            pressed_actions_movement: "neutral".to_string(),
            pressed_actions_basic: vec![],
            pressed_actions_composite: vec![]
        }
    }
}

pub struct FrameInputState {
    pub movement: String,
    pub basic_actions: Vec<String>,
    pub composite_actions: Vec<String>,
    pub charge: HashMap<String, i32>
}

impl Default for FrameInputState {
    fn default() -> Self {
        Self {
            movement: "neutral".to_string(),
            basic_actions: vec![],
            composite_actions: vec![],
            charge: HashMap::new()
        }
    }
}
