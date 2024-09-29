/// A [`Requirement`] is a condition that must be satisfied for a heuristic to
/// be applicable in a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Requirement {
    NoNegativePreconditions,
}
