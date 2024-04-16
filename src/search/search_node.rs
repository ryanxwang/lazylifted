use crate::search::{HeuristicValue, StateId, Transition, NO_STATE};
use ordered_float::Float;

/// The status of a search node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchNodeStatus {
    /// New node, not yet opened
    New,
    /// Node is in the open list
    Open,
    /// Node is in the closed list
    Closed,
    /// Node is a deadend
    Deadend,
}

/// A [`SearchNode`] is a node in the search space. It contains information
/// about the state specific to the search, such as heuristic cost and parent
/// node.
#[derive(Debug, Clone)]
pub struct SearchNode<T>
where
    T: Transition,
{
    /// Unique identifier of the state
    state_id: StateId,
    /// Status of the node
    status: SearchNodeStatus,
    /// F-value of the node, different depending on the search algorithm.
    f: HeuristicValue,
    /// G-value of the node, i.e. the cost to reach this node. In search
    /// algorithms that only care about the f-value, this should be ignored.
    g: HeuristicValue,
    /// H-value of the node, i.e. the heuristic estimate of the cost to reach
    /// the goal. In search algorithms that only care about the f-value, this
    /// should be ignored.
    h: HeuristicValue,
    /// Transition that led to this node
    transition: T,
    /// Parent state
    parent_id: StateId,
    /// Whether the node is preferred by the parent node
    is_preferred: bool,
}

impl<T> SearchNode<T>
where
    T: Transition,
{
    /// Create a new search node with no parent. This should only be used for
    /// the root node of the search space. For non-root nodes see
    /// [`SearchNode::new_with_parent`].
    pub fn new_without_parent() -> Self {
        Self {
            state_id: StateId::new(),
            status: SearchNodeStatus::New,
            f: HeuristicValue::infinity(),
            g: HeuristicValue::infinity(),
            h: HeuristicValue::infinity(),
            transition: T::no_transition(),
            parent_id: NO_STATE,
            is_preferred: false,
        }
    }

    /// Create a new search node with a parent. This should be used for all
    /// nodes that are not the root node. For root nodes see
    /// [`SearchNode::new_without_parent`].
    pub fn new_with_parent(parent_id: StateId, transition: T) -> Self {
        Self {
            state_id: StateId::new(),
            status: SearchNodeStatus::New,
            f: HeuristicValue::infinity(),
            g: HeuristicValue::infinity(),
            h: HeuristicValue::infinity(),
            transition,
            parent_id,
            is_preferred: false,
        }
    }

    pub fn set_is_preferred(&mut self, is_preferred: bool) {
        self.is_preferred = is_preferred;
    }

    pub fn open(&mut self, g: HeuristicValue, h: HeuristicValue) {
        self.status = SearchNodeStatus::Open;
        self.g = g;
        self.h = h;
        self.f = g + h;
    }

    pub fn open_with_f(&mut self, f: HeuristicValue) {
        self.status = SearchNodeStatus::Open;
        self.f = f;
    }

    pub fn mark_as_deadend(&mut self) {
        self.status = SearchNodeStatus::Deadend;
        self.f = HeuristicValue::infinity();
    }

    pub fn close(&mut self) {
        debug_assert_eq!(
            self.status,
            SearchNodeStatus::Open,
            "Node must be open to close it"
        );
        self.status = SearchNodeStatus::Closed;
    }

    pub fn get_status(&self) -> SearchNodeStatus {
        self.status
    }

    pub fn get_state_id(&self) -> StateId {
        self.state_id
    }

    pub fn get_f(&self) -> HeuristicValue {
        self.f
    }

    pub fn get_g(&self) -> HeuristicValue {
        self.g
    }

    pub fn get_h(&self) -> HeuristicValue {
        self.h
    }

    pub fn get_parent_id(&self) -> StateId {
        self.parent_id
    }

    pub fn get_transition(&self) -> &T {
        &self.transition
    }

    pub fn is_preferred(&self) -> bool {
        self.is_preferred
    }
}
