pub enum ActionType {
    Basic {
        input_action: String,
        is_dependency: bool
    },

    Composite {
        dependencies: Vec<String>,
        require_all: bool
    }
}

pub struct Action {
    pub pressed: bool,
    pub charge_type: u32,
    pub action_type: ActionType
}

impl Action {
    pub fn basic(input_action: impl Into<String>, charge_type: u32) -> Self {
        Self {
            pressed: false,
            charge_type,
            action_type: ActionType::Basic {
                input_action: input_action.into(),
                is_dependency: false
            }
        }
    }

    pub fn composite(dependencies: Vec<impl Into<String>>, charge_type: u32, require_all: bool) -> Self {
        Self {
            pressed: false,
            charge_type,
            action_type: ActionType::Composite {
                dependencies: dependencies.into_iter().map(|d| d.into()).collect(),
                require_all
            }
        }
    }
}
