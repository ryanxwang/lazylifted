#[derive(Debug, Clone)]
pub struct TransformationOptions {
    #[allow(dead_code)]
    pub rename_variables: bool,
    #[allow(dead_code)]
    pub collapse_predicates: bool,
    pub remove_action_predicates: bool,
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
            remove_action_predicates: remove_action_predicate,
        }
    }
}

impl Default for TransformationOptions {
    fn default() -> Self {
        Self {
            rename_variables: true,
            collapse_predicates: true,
            remove_action_predicates: true,
        }
    }
}
