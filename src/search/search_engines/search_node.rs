use crate::search::{
    action::NO_ACTION,
    search_engines::{StateId, NO_STATE},
    Action, HeuristicValue,
};
use ordered_float::Float;

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
    f: HeuristicValue,
    /// G-value of the node, i.e. the cost to reach this node. In search
    /// algorithms that only care about the f-value, this should be ignored.
    g: HeuristicValue,
    /// H-value of the node, i.e. the heuristic estimate of the cost to reach
    /// the goal. In search algorithms that only care about the f-value, this
    /// should be ignored.
    h: HeuristicValue,
    /// Action that led to this node
    action: Action,
    /// Parent state
    parent_id: StateId,
    /// Whether the node is preferred by the parent node
    is_preferred: bool,
}

impl SearchNode {
    pub fn new_without_parent() -> Self {
        Self {
            state_id: StateId::new(),
            status: SearchNodeStatus::New,
            f: HeuristicValue::infinity(),
            g: HeuristicValue::infinity(),
            h: HeuristicValue::infinity(),
            action: NO_ACTION,
            parent_id: NO_STATE,
            is_preferred: false,
        }
    }

    pub fn new_with_parent(parent_id: StateId, action: Action) -> Self {
        Self {
            state_id: StateId::new(),
            status: SearchNodeStatus::New,
            f: HeuristicValue::infinity(),
            g: HeuristicValue::infinity(),
            h: HeuristicValue::infinity(),
            action,
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

    pub fn get_action(&self) -> &Action {
        &self.action
    }

    pub fn is_preferred(&self) -> bool {
        self.is_preferred
    }
}
