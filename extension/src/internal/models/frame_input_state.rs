use std::collections::HashMap;

#[derive(Default, Clone)]
pub struct FrameInputState {
    pub movement: String,
    pub basic_actions: Vec<String>,
    pub composite_actions: Vec<String>,
    pub all: String,
    pub charge: HashMap<String, u32>,
}
