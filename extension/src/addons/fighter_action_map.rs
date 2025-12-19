use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct FighterActionMap {
    #[export]
    pub up: GString,

    #[export]
    pub down: GString,

    #[export]
    pub forward: GString,

    #[export]
    pub back: GString,

    #[export]
    pub actions: Array<Gd<FighterAction>>,

    #[export]
    pub composite_actions: Array<Gd<FighterCompositeAction>>,

    #[export(range = (0.0, 60.0, or_greater))]
    pub charge_requirement: i32,

    base: Base<Resource>,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct FighterAction {
    #[export]
    pub name: GString,

    #[export]
    pub input_action: GString,

    #[export(enum = (None = 0, Immediate = 1, Tick = 2))]
    pub charge_type: i32
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
pub struct FighterCompositeAction {
    #[export]
    pub name: GString,

    #[export]
    pub dependencies: Array<GString>,

    #[export]
    pub require_all: bool,

    #[export(enum = (None = 0, Immediate = 1, Tick = 2))]
    pub charge_type: i32
}
