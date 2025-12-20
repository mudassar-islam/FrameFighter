use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=Object)]
pub struct FrameFighter {
    base: Base<Object>
}

#[godot_api]
impl FrameFighter {
    #[constant]
    pub const CHARGE_NONE: i32 = 0;
    #[constant]
    pub const CHARGE_IMMEDIATE: i32 = 1;

    #[constant]
    pub const CHARGE_TICK: i32 = 2;

    #[constant]
    pub const PLAYER_ONE: i32 = 1;

    #[constant]
    pub const PLAYER_TWO: i32 = 2;

    #[constant]
    pub const ACCURACY_PERFECT: i32 = 0;

    #[constant]
    const ACCURACY_LENIENT: i32 = 1;

    #[constant]
    pub const PERFECT_INPUT: i32 = 0;
    #[constant]
    pub const LENIENT_FINAL_INPUT: i32 = 1;
    #[constant]
    pub const DIAGONALS_OPTIONAL: i32 = 2;
    #[constant]
    pub const NEUTRALS_REQUIRED: i32 = 3;

    pub fn is_movement(name: &str) -> bool {
        matches!(
            name,
            "up" | "down" | "back" | "forward"
                | "up_forward" | "up_back"
                | "down_forward" | "down_back"
        )
    }
}
