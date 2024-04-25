use crate::search::{PartialAction, Task, Transition};

/// Action struct that represents an instantiated action schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// The action schema index.
    pub index: usize,
    /// List of object indices that instantiate the action schema.
    pub instantiation: Vec<usize>,
}

impl Action {
    pub fn new(index: usize, instantiation: Vec<usize>) -> Self {
        Self {
            index,
            instantiation,
        }
    }

    pub fn from_partial(partial: &PartialAction, task: &Task) -> Option<Self> {
        let schema = &task.action_schemas()[partial.schema_index()];
        if partial.partial_instantiation().len() != schema.parameters().len() {
            return None;
        }

        Some(Self {
            index: partial.schema_index(),
            instantiation: partial.partial_instantiation().to_vec(),
        })
    }

    pub fn to_string(&self, task: &Task) -> String {
        let schema_name: &str = task.action_schemas()[self.index].name().as_ref();
        let objects: Vec<&str> = self
            .instantiation
            .iter()
            .map(|&object_index| task.objects[object_index].name.as_ref())
            .collect();

        format!("({} {})", schema_name, objects.join(" "))
    }
}

/// This is a hack specific to the initial node of the search space, which does
/// not have a parent action. [`NO_ACTION`] should only be used for this
/// purpose. We use this instead of an [`Option<Action>`] to avoid the overhead
/// of an [`Option`] type.
const NO_ACTION: Action = Action {
    index: usize::MAX,
    instantiation: vec![],
};

impl Transition for Action {
    fn no_transition() -> Self {
        NO_ACTION
    }
}
