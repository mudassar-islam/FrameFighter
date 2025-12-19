use std::{collections::HashMap};
use godot::{classes::Input, global::{godot_error, godot_print}, obj::Singleton};
use indexmap::IndexMap;

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
    actions: IndexMap<String, Action>,
    charge: HashMap<String, i32>,
    opposites: HashMap<String, (String, String)>,
    dependency_input: HashMap<String, bool>,

    can_charge: bool,

    pressed_actions_movement: String,
    pressed_actions_basic: Vec<String>,
    pressed_actions_composite: Vec<String>,
}

impl ActionController {
    pub fn bind_directions(&mut self, up: String, down: String, forward: String, back: String, charge_type: i32) {
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
        self.create_charge_key("up", 0);
        self.create_charge_key("down", 0);
        self.create_charge_key("back", 0);
        self.create_charge_key("forward", 0);

        // A hashmap for the opposite action and opposite input action for every direction.
        // Required for SOCD checks.
        self.opposites.insert("up".to_string(), ("down".to_string(), down.to_string()));
        self.opposites.insert("down".to_string(), ("up".to_string(), up.to_string()));
        self.opposites.insert("back".to_string(), ("forward".to_string(), forward.to_string()));
        self.opposites.insert("forward".to_string(), ("back".to_string(), back.to_string()));
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
    }

    pub fn create_charge_key(&mut self, name: &str, frames: i32) {
        self.charge.insert(name.to_string(), frames);
    }

    pub fn should_charge(&mut self, can_charge: bool) {
        self.can_charge = can_charge;
    }

    pub fn handle_side(&mut self, side: i32) {
        if let Some(forward) = self.opposites.get("back") && let Some(back) = self.opposites.get("forward") {
            godot_print!("{} {}", &forward.1, &back.1);

            if let Some(forward_action) = self.actions.get_mut("forward") {
                forward_action.input_action = if side == FrameFighter::PLAYER_ONE {
                    forward.1.to_string()
                } else {
                    back.1.to_string()
                };
            }

            if let Some(back_action) = self.actions.get_mut("back") {
                back_action.input_action = if side == FrameFighter::PLAYER_ONE {
                    back.1.to_string()
                } else {
                    forward.1.to_string()
                };
            }
        }
    }

    pub fn process_current_frame(&mut self) {
        let input = Input::singleton();

        // Actions pushed into this vec will be "released" after all iterations are complete.
        let mut released_actions = vec![];

        for (_name, action) in self.actions.iter_mut() {
            match action.composite {
                true => {
                    let mut dependency_state = action.dependencies.iter()
                        .map(|dependency| if let Some(&is_true) = self.dependency_input.get(&dependency.to_string()) && is_true {
                            true
                        } else {
                            false
                        });

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
                        match self.opposites.get(&action.name) {
                            Some((opposite_action, opposite_input)) => {
                                if input.is_action_pressed(opposite_input) {
                                    action.pressed = false;
                                    released_actions.push(opposite_action.clone());
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
            can_charge: false,

            actions: IndexMap::new(),
            charge: HashMap::new(),
            opposites: HashMap::new(),
            dependency_input: HashMap::new(),

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
