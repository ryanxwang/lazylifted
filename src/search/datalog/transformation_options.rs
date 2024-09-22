#[derive(Debug, Clone)]
pub struct TransformationOptions {
    pub rename_variables: bool,
    pub collapse_predicates: bool,
    pub remove_action_predicate: bool,
}

impl TransformationOptions {
    pub fn new(
        rename_variables: bool,
        collapse_predicates: bool,
        remove_action_predicate: bool,
    ) -> Self {
        Self {
            rename_variables,
            collapse_predicates,
            remove_action_predicate,
        }
    }
}

impl Default for TransformationOptions {
    fn default() -> Self {
        Self {
            rename_variables: true,
            collapse_predicates: true,
            remove_action_predicate: true,
        }
    }
}
