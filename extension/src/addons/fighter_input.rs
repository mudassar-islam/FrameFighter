use godot::prelude::*;

use crate::addons::fighter_history::{FighterHistoryItem, FighterMatchedMove};
use crate::addons::frame_fighter::FrameFighter;
use crate::addons::fighter_action_map::FighterActionMap;
use crate::addons::fighter_move_list::FighterMoveList;
use crate::internal::action_controller::{ActionController, FrameInputState};
use crate::internal::input_history::InputHistory;
use crate::internal::move_matcher::{MoveMatcher, SequenceStep};

#[derive(GodotClass)]
#[class(tool, init, base=Node)]
/// A node for fighting game input processing. Evaluates input, tracks input history, and compares input to a move-list to resolve valid moves.
///
/// **Note:** A [FighterActionMap] & [FighterMoveList] resource is required for basic function.
struct FighterInput {
    /// Set the amount of entries the input history will hold. This number determines the largest possible sequence that can be matched. The default value of 20 should be enough for most use-cases.
    #[export]
    #[init(val = 20)]
    history_size: u32,

    /// Set the amount of frames the input history ticks up-to. The default value of 99 should be enough for most use-cases.
    #[export]
    #[init(val = 99)]
    history_frames: u32,

    #[export]
    action_map: Option<Gd<FighterActionMap>>,
    #[export]
    move_list: Option<Gd<FighterMoveList>>,

    frame_input_state: FrameInputState,
    side: u32,
    can_charge: bool,
    action_controller: ActionController,
    move_matcher: MoveMatcher,
    input_history: InputHistory
}

#[godot_api]
impl INode for FighterInput {
    fn ready(&mut self) {
        match usize::try_from(self.history_size) {
            Ok(size) => {
                self.input_history.set_size(size);
                self.move_matcher.set_size(size);
            },
            Err(_) => {
                self.input_history.set_size(20);
                self.move_matcher.set_size(20);

                godot_error!("History size could not be converted into usize in the FighterInput node. Using the default value of 20.");
            }
        }

        self.input_history.set_max_frames(self.history_frames);
        self.move_matcher.set_max_frames(self.history_frames);

        self.setup_action_controller(); // Bind actions to the ActionController from the FighterActionMap resource.
        self.setup_move_matcher();      // Bind moves to the MoveMatcher from the FighterMoveList resource.
    }
}

#[godot_api]
impl FighterInput {
    /// Set the player side to Player 1 or Player 2. Inverts the forward & back actions for opposing sides.
    #[func]
    pub fn set_side(&mut self, side: u32) {
        self.side = side;
    }

    /// Set whether actions can be charged or not. For example, actions should be charged while on the ground but not in the air.
    #[func]
    pub fn set_can_charge(&mut self, can_charge: bool) {
        self.can_charge = can_charge;
    }

    /// Evaluates the current frame & returns a [FighterHistoryItem]
    #[func]
    pub fn process_frame(&mut self) -> Gd<FighterHistoryItem> {
        self.action_controller.handle_side(self.side);
        self.action_controller.set_can_charge(self.can_charge);
        self.action_controller.process_frame();

        self.frame_input_state = self.action_controller.get_frame_state();

        self.input_history.add(&self.frame_input_state);
        self.move_matcher.add_buffer_entry(&self.frame_input_state);

        self.move_matcher.process_frame();

        // self.log_input_state();

        /* for move_item in matched_moves.iter() {
            godot_print!("{} - {} - {}", &move_item.name, &move_item.perfect_input, &move_item.total_frames)
        } */

        Gd::from_object(self.input_history.pressed_actions_for_godot())
    }

    /// Get the input history for display. The input history is an array that contains active actions and pressed for duration in frames. Ordered by most recent entry to oldest entry.
    ///
    /// **Note:** Must be called after **process_frame()** to retrieve the an up-to-date result.
    #[func]
    pub fn history(&mut self) -> Array<Gd<FighterHistoryItem>> {
        self.input_history.to_godot()
    }

    /// [InputHistoryItem] for the current frame.
    #[func]
    fn pressed_actions(&self) -> Gd<FighterHistoryItem> {
        Gd::from_object(self.input_history.pressed_actions_for_godot())
    }

    /// Array of MatchedMoves for the current frame.
    #[func]
    fn matched_moves(&mut self) -> Array<Gd<FighterMatchedMove>> {
        self.move_matcher.matched_moves_for_godot()
    }

    fn setup_move_matcher(&mut self) {
        match &self.move_list {
            Some(move_list) => {
                let move_list = move_list.bind();

                for fighter_move in move_list.moves.iter_shared() {
                    let fighter_move = fighter_move.bind();

                    let steps = fighter_move.sequence.iter_shared()
                        .map(|s| {
                            let s = s.bind();
                            SequenceStep::new(
                                s.movement.to_string(),
                                s.actions.iter_shared().collect(),
                                s.input_window,
                                s.modifiers.iter_shared().collect()
                            )
                        })
                        .collect();

                    self.move_matcher.add_move(
                        fighter_move.name.to_string(),
                        steps,
                        fighter_move.require_charge,
                        fighter_move.charge_frames
                    );
                }

                self.move_matcher.sort_moves();

                // self.move_matcher.build_sequence_string();
            },

            None => godot_error!("Fighter Move List not bound to FighterInput Node.")
        }
    }

    fn setup_action_controller(&mut self) {
        match &self.action_map {
            Some(action_map) => {
                let action_map = action_map.bind();

                let (
                    up,
                    down,
                    forward,
                    back
                ) = (
                    action_map.up.to_string(),
                    action_map.down.to_string(),
                    action_map.forward.to_string(),
                    action_map.back.to_string()
                );

                // Basic actions for movement. Can all be charged.
                self.action_controller.add("up", &up, FrameFighter::CHARGE_IMMEDIATE);
                self.action_controller.add("down", &down, FrameFighter::CHARGE_IMMEDIATE);
                self.action_controller.add("forward", &forward, FrameFighter::CHARGE_IMMEDIATE);
                self.action_controller.add("back", &back, FrameFighter::CHARGE_IMMEDIATE);

                // Composite actions for diagonal movement. Cannot be charged and require all dependencies to be pressed
                self.action_controller.add_composite("up_forward", vec![ "up", "forward" ], FrameFighter::CHARGE_NONE, true);
                self.action_controller.add_composite("down_forward", vec![ "down", "forward" ], FrameFighter::CHARGE_NONE, true);
                self.action_controller.add_composite("up_back", vec![ "up", "back" ], FrameFighter::CHARGE_NONE, true);
                self.action_controller.add_composite("down_back", vec![ "down", "back" ], FrameFighter::CHARGE_NONE, true);

                // A hashmap for the opposite action and opposite input action for every direction.
                // Required for SOCD checks.
                self.action_controller.build_opposite_map("up", "down", &down, &down);
                self.action_controller.build_opposite_map("down", "up", &up, &up);
                self.action_controller.build_opposite_map("back", "forward", &forward, &forward);
                self.action_controller.build_opposite_map("forward", "back", &back, &back);

                // Bind Basic Actions
                for action in action_map.actions.iter_shared() {
                    let action = action.bind();

                    self.action_controller.add(
                        action.name.to_string(),
                        action.input_action.to_string(),
                        action.charge_type
                    );
                }

                // Bind Composite Actions
                for action in action_map.composite_actions.iter_shared() {
                    let action = action.bind();

                    let dependencies: Vec<String> = action.dependencies.iter_shared().map(|d| d.to_string()).collect();

                    self.action_controller.add_composite(
                        action.name.to_string(),
                        dependencies.iter().map(|s| s.as_str()).collect(),
                        action.charge_type,
                        action.require_all
                    );
                }

                self.action_controller.build_charge_map();
                self.action_controller.build_dependency_map();
            },

            None => godot_error!("Fighter Action Map not bound to FighterInput Node.")
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
