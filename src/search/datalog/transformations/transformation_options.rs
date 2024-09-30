#[derive(Debug, Clone)]
pub struct TransformationOptions {
    // TODO-soon: implement these options
    #[allow(dead_code)]
    pub rename_variables: bool,
    #[allow(dead_code)]
    pub collapse_predicates: bool,
    pub remove_action_predicates: bool,
    /// This should only be true for partial space search.
    pub restrict_immediate_applicability: bool,
}

impl TransformationOptions {
    #[allow(dead_code)]
    pub fn new(
        rename_variables: bool,
        collapse_predicates: bool,
        remove_action_predicates: bool,
        restrict_immediate_applicability: bool,
    ) -> Self {
        Self {
            rename_variables,
            collapse_predicates,
            remove_action_predicates,
            restrict_immediate_applicability,
        }
    }

    pub fn with_restrict_immediate_applicability(mut self) -> Self {
        self.restrict_immediate_applicability = true;
        self
    }
}

impl Default for TransformationOptions {
    fn default() -> Self {
        Self {
            rename_variables: true,
            collapse_predicates: true,
            remove_action_predicates: true,
            restrict_immediate_applicability: false,
        }
    }
}
