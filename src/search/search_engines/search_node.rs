use crate::search::{
    action::NO_ACTION,
    search_engines::{StateId, NO_STATE},
    Action,
};

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

#[derive(Debug, Clone)]
pub struct SearchNode {
    /// Unique identifier of the state
    state_id: StateId,
    /// Status of the node
    status: SearchNodeStatus,
    /// F-value of the node, different depending on the search algorithm.
    f: f64,
    /// G-value of the node, i.e. the cost to reach this node. In search
    /// algorithms that only care about the f-value, this should be ignored.
    g: f64,
    /// H-value of the node, i.e. the heuristic estimate of the cost to reach
    /// the goal. In search algorithms that only care about the f-value, this
    /// should be ignored.
    h: f64,
    /// Action that led to this node
    action: Action,
    /// Parent state
    parent_id: StateId,
}

impl SearchNode {
    pub fn new_without_parent() -> Self {
        Self {
            state_id: StateId::new(),
            status: SearchNodeStatus::New,
            f: f64::INFINITY,
            g: f64::INFINITY,
            h: f64::INFINITY,
            action: NO_ACTION,
            parent_id: NO_STATE,
        }
    }

    pub fn new_with_parent(parent_id: StateId, action: Action) -> Self {
        Self {
            state_id: StateId::new(),
            status: SearchNodeStatus::New,
            f: f64::INFINITY,
            g: f64::INFINITY,
            h: f64::INFINITY,
            action,
            parent_id,
        }
    }

    pub fn open(&mut self, g: f64, h: f64) {
        self.status = SearchNodeStatus::Open;
        self.g = g;
        self.h = h;
        self.f = g + h;
    }

    pub fn open_with_f(&mut self, f: f64) {
        self.status = SearchNodeStatus::Open;
        self.f = f;
    }

    pub fn mark_as_deadend(&mut self) {
        self.status = SearchNodeStatus::Deadend;
        self.f = f64::INFINITY;
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

    pub fn get_f(&self) -> f64 {
        self.f
    }
}
