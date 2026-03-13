use godot::prelude::*;

/// Includes some constants for use with the [FighterInput] node.
#[derive(GodotClass)]
#[class(tool, init, base=Object)]
pub struct FrameFighter {}

#[godot_api]
impl FrameFighter {
    /// Player 1 Side. Default side. Sequences are defined within the context of Player 1.
    #[constant]
    pub const PLAYER_ONE: u32 = 1;
    /// Player 2 Side. Forward & Back are inverted.
    #[constant]
    pub const PLAYER_TWO: u32 = 2;

    /* Move Step Modifiers */
    #[constant]
    pub const LENIENT_ENDER: u32 = 1;
    #[constant]
    pub const IGNORE_DIAGONAL: u32 = 2;

    /* Types of Charge
     * --------------- */
    /// Will not be charged.
    #[constant]
    pub const CHARGE_NONE: u32 = 0;
    /// Charge is increased once per tick, reset to zero when released.
    #[constant]
    pub const CHARGE_IMMEDIATE: u32 = 1;
    /// Charge is increased once per tick, reduced once per tick when released.
    #[constant]
    pub const CHARGE_TICK: u32 = 2;

    pub fn is_movement(name: &str) -> bool {
        matches!(
            name,
            "up" | "down" | "back" | "forward" | "up_forward" | "up_back" | "down_forward" | "down_back"
        )
    }

    pub fn is_cardinal(name: &str) -> bool {
        matches!(
            name,
            "up" | "down" | "back" | "forward"
        )
    }

    // For a given transition between cardinal to a diagonal, it gives you the expected ending direction for the sequence.
    // For example, if a move goes from down to down_forward, the next expected input is forward.
    pub fn expected_ender(dir_a: &str, dir_b: &str) -> String {
        match (dir_a, dir_b) {
            ("down", "down_forward") => "forward".to_string(),
            ("down", "down_back") => "back".to_string(),

            ("up", "up_forward") => "forward".to_string(),
            ("up", "up_back") => "back".to_string(),

            ("forward", "down_forward") => "down".to_string(),
            ("forward", "up_forward") => "up".to_string(),

            ("back", "down_back") => "down".to_string(),
            ("back", "up_back") => "up".to_string(),
            _ => "neutral".to_string()
        }
    }
}
