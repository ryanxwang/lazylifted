use crate::search::{NodeId, Plan, SearchNode, Transition};

/// A [`SearchProblem`] is a problem formulation exposing the necessary
/// information to the search algorithms. It allows framing the planning search
/// problem in different ways, outside of the canonical state space search
/// formulation. Implementations of this trait should also implement various
/// logging and statistics collection mechanisms. See
/// [`StateSpaceProblem`](crate::search::problem_formulations::StateSpaceProblem)
/// for an example.
pub trait SearchProblem<S, T: Transition> {
    /// Returns the initial state of the search problem.
    fn initial_state(&self) -> &SearchNode<T>;

    /// Returns whether the given state is a goal state.
    fn is_goal(&self, node_id: NodeId) -> bool;

    /// Expand the given state and returns a list of references to the expanded
    /// nodes. If the node is already closed, it returns an empty list,
    /// otherwise it closes the node.
    fn expand(&mut self, node_id: NodeId) -> Vec<&SearchNode<T>>;

    /// Extracts the plan from the given goal node. Will also mark the end of
    /// the search. This should be called only once. When this is called, final
    /// statistics logs are emitted. No further calls to
    /// [`SearchProblem::expand`] should be made.
    fn extract_plan(&self, goal_id: NodeId) -> Plan;
}
