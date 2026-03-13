#[derive(Clone)]
pub struct SequenceStep {
    pub movement: String,
    pub actions: Vec<String>,
    pub input_window: u32,
    pub modifiers: Vec<u32>,
}

impl SequenceStep {
    pub fn new(
        movement: impl Into<String>,
        actions: Vec<impl Into<String>>,
        input_window: u32,
        modifiers: Vec<u32>,
    ) -> Self {
        Self {
            movement: movement.into(),
            actions: actions.into_iter().map(|a| a.into()).collect(),
            input_window,
            modifiers,
        }
    }
}
