use godot::prelude::*;

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
/// A resource that tells the [FighterInput] node how to evaluate player input.
///
/// Set the corresponding **Input Map Action** for every action your game requires.
/// The movement actions are pre-set only require the **Input Map Action** name, however other actions can be created manually and be tailored specifically to your project's needs.
///
/// **Note:** All movement actions can be charged.
pub struct FighterActionMap {
    /// The Input Map Action for the up direction.
    #[export]
    pub up: GString,

    /// The Input Map Action for the down direction.
    #[export]
    pub down: GString,

    /// The Input Map Action for the forward direction.
    #[export]
    pub forward: GString,

    /// The Input Map Action for the back direction.
    #[export]
    pub back: GString,

    /// Array of custom [FighterAction]s
    #[export]
    pub actions: Array<Gd<FighterAction>>,

    /// Array of custom [FighterCompositeAction]s
    #[export]
    pub composite_actions: Array<Gd<FighterCompositeAction>>,

    #[export(range = (0.0, 60.0, or_greater))]
    pub charge_requirement: i32,

    base: Base<Resource>,
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
/// A manually defined action that can be evaluated by [FighterInput]
pub struct FighterAction {
    /// A name for this [FighterAction] i.e. lp for Light Punch.
    #[export]
    pub name: GString,

    /// The corresponding **Input Map Action** from your project settings.
    #[export]
    pub input_action: GString,

    /// What type of charge action uses. Cannot be charged by default.
    #[export(enum = (None = 0, Immediate = 1, Tick = 2))]
    pub charge_type: i32
}

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
/// A composite action that can be evaluated by [FighterInput]. Depends on pre-existing [FighterAction]s.
pub struct FighterCompositeAction {
    #[export]
    /// A name for this [FighterCompositeAction] i.e. "ex" for EX Punch (Holding lp + mp + hp)
    pub name: GString,

    /// Array of names of [FighterAction]s this **FighterCompositeAction** depends upon.
    #[export]
    pub dependencies: Array<GString>,

    /// Whether all dependenices need to be pressed to activate this action.
    ///
    /// **Example:** An EX Punch may depend on lp + mp + hp, but should all buttons be pressed for it to be activated or should it also be activated for just lp + mp?
    #[export]
    pub require_all: bool,

    /// What type of charge action uses. Cannot be charged by default.
    #[export(enum = (None = 0, Immediate = 1, Tick = 2))]
    pub charge_type: i32
}
