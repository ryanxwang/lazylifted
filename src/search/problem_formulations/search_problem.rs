use crate::search::{SearchNode, StateId, Transition};

pub trait SearchProblem<S, T>
where
    T: Transition,
{
    fn initial_state(&self) -> &SearchNode<T>;

    fn is_goal(&self, state_id: StateId) -> bool;

    fn expand(&mut self, state_id: StateId) -> Vec<&SearchNode<T>>;
}
