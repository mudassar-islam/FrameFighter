use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
/// A collection of moves that can be evaluated by [FighterInput]
pub struct FighterMoveList {
    #[export]
    /// Global input window for this whole move-list. The next input needs to be pressed within this frame-window for the move to be activated.
    /// Can be controlled on a per-move basis inside [FighterMove]
    pub input_window: u32,

    #[export]
    /// Array of [FighterMove]
    pub moves: Array<Gd<FighterMove>>
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
/// An individual FighterMove that can be matched by [FighterInput]. Has a sequence that needs to be input for it to be activated. Modifiers can be used to change how the sequence is matched.
pub struct FighterMove {
    #[export]
    /// The name of this move.
    pub name: GString,

    #[export]
    /// The priority for this move. Determines which move is activated if multiple moves are matched on the same frame. Lower number means higher priority.
    pub priority: i32,

    #[export]
    /// Whether this move requires neutrals. Usually you should keep this off unless necessary for a specific move. #[export]
    pub require_neutrals: bool,

    /// Whether the first input for this move requires charge.
    #[export]
    pub require_charge: bool,

    /// Applies only to charge moves. Determines how many frames to wait before a charge move is allowed to activate.
    #[export]
    pub charge_frames: u32,

    #[export]
    /// The sequence that needs to be input for this move to be activated.
    pub sequence: Array<Gd<FighterSequenceStep>>
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
/// An individual step for a sequence.
pub struct FighterSequenceStep {
    /// The movement direction that needs to be pressed on this step.
    #[export]
    pub movement: GString,

    /// The actions that need to be pressed for this step.
    #[export]
    pub actions: Array<GString>,

    /// Modifiers allow flexibility in how moves are matched. They can allow for leniency.
    #[export]
    pub modifiers: Array<u32>,

    /// The next input needs to be pressed within this frame window to activate this move.
    #[export]
    pub input_window: u32
}
