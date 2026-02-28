use godot::prelude::*;

/// Includes some constants for use with the [FighterInput] node.
#[derive(GodotClass)]
#[class(tool, init, base=Object)]
pub struct FrameFighter {
    base: Base<Object>
}

#[godot_api]
impl FrameFighter {
    /// Will not be charged.
    #[constant]
    pub const CHARGE_NONE: i32 = 0;

    /// Charge is increased once per tick, reset to zero when released.
    #[constant]
    pub const CHARGE_IMMEDIATE: i32 = 1;

    /// Charge is increased once per tick, reduced once per tick when released.
    #[constant]
    pub const CHARGE_TICK: i32 = 2;

    /// Player 1 Side. Default side. Sequences are defined within the context of Player 1.
    #[constant]
    pub const PLAYER_ONE: i32 = 1;

    /// Player 2 Side. Forward & Back are inverted.
    #[constant]
    pub const PLAYER_TWO: i32 = 2;

    pub fn is_movement(name: &str) -> bool {
        matches!(
            name,
            "up" | "down" | "back" | "forward" | "up_forward" | "up_back" | "down_forward" | "down_back"
        )
    }
}
