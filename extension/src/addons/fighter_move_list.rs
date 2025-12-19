use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct FighterMoveList {
    #[export]
    moves: Array<Gd<FighterMove>>,
    base: Base<Resource>,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct FighterMove {
    #[export]
    name: GString,

    #[export]
    priority: i32,

    #[export]
    sequence: Array<GString>,

    #[export]
    modifiers: Array<i32>,

    #[export]
    frame_buffer: i32
}
