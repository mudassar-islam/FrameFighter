use crate::internal::models::sequence_step::SequenceStep;

pub struct Move {
    pub reversed_sequence: Vec<SequenceStep>,
    pub require_neutrals: bool,
    pub require_charge: bool,
    pub charge_frames: u32,
    pub priority: i32
}

impl Move {
    pub fn new(sequence: Vec<SequenceStep>, require_neutrals: bool, require_charge: bool, charge_frames: u32, priority: i32) -> Self {
        let mut reversed_sequence = sequence.clone();
        reversed_sequence.reverse();

        Self {
            reversed_sequence,
            require_neutrals,
            require_charge,
            charge_frames,
            priority
        }
    }
}

pub struct MatchedMove {
    pub name: String,
    pub priority: i32,
    pub total_frames: u32,
    pub perfect_input: bool
}
