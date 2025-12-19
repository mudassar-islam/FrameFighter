use godot::prelude::*;

use crate::addons::frame_fighter::FrameFighter;
use crate::addons::fighter_action_map::FighterActionMap;
use crate::addons::fighter_move_list::FighterMoveList;
use crate::internal::action_controller::{ActionController, FrameInputState};

#[derive(GodotClass)]
#[class(tool, init, base=Node)]
struct FighterInput {
    pub frame_input_state: FrameInputState,

    side: i32,
    can_charge: bool,
    action_controller: ActionController,

    /* Resources for configuration, Exposed to the editor */
    #[export]
    action_map: Option<Gd<FighterActionMap>>,
    #[export]
    move_list: Option<Gd<FighterMoveList>>,
    /* --- */

    base: Base<Node>
}

#[godot_api]
impl INode for FighterInput {
    fn ready(&mut self) {
        self.parse_action_map(); // Bind actions locally from the Fighter Action Map resource.
    }
}

#[godot_api]
impl FighterInput {
    #[func]
    pub fn set_side(&mut self, side: i32) {
        self.side = side;
    }

    #[func]
    pub fn should_charge(&mut self, can_charge: bool) {
        self.can_charge = can_charge;
    }

    #[func]
    pub fn tick(&mut self) {
        self.action_controller.handle_side(self.side);
        self.action_controller.should_charge(self.can_charge);
        self.action_controller.process_current_frame();
        self.frame_input_state = self.action_controller.frame_input_state();

        self.log_input_state();
    }

    #[func]
    pub fn get_history() -> bool {
        true
    }

    #[func]
    pub fn get_actions() -> bool {
        true
    }

    fn parse_action_map(&mut self) {
        match &self.action_map {
            Some(action_map) => {
                let action_map = action_map.bind();

                self.action_controller.bind_directions(
                    action_map.up.to_string(),
                    action_map.down.to_string(),
                    action_map.forward.to_string(),
                    action_map.back.to_string(),

                    FrameFighter::CHARGE_IMMEDIATE
                );

                for action in action_map.actions.iter_shared() {
                    let action = action.bind();
                    self.action_controller.add(&action.name.to_string(), &action.input_action.to_string(), action.charge_type);
                }

                for action in action_map.composite_actions.iter_shared() {
                    let action = action.bind();

                    let mut dependencies: Vec<String> = vec![];

                    for dependency in action.dependencies.iter_shared() {
                        dependencies.push(dependency.to_string());
                    }

                    self.action_controller.add_composite(
                        &action.name.to_string(),
                        dependencies.iter().map(|s| s.as_str()).collect(),
                        action.charge_type,
                        action.require_all
                    );
                }
            },

            None => panic!("Fighter Action Map not found.")
        }
    }

    fn log_input_state(&self) {
        let mut log_string = String::from("\n \nMovement: ");

        log_string += &self.frame_input_state.movement;
        log_string += "\nBasic: [ ";
        log_string += &self.frame_input_state.basic_actions.join(", ");
        log_string += " ] - Composite: [ ";
        log_string += &self.frame_input_state.composite_actions.join(", ");
        log_string += " ]\n";
        log_string += "Charge: { ";

        for (action, charge) in self.frame_input_state.charge.iter() {
            log_string += action;
            log_string += ": ";
            log_string += &charge.to_string();
            log_string += ", ";
        }

        log_string += "}";

        godot_print!("{}", log_string);
    }
}
