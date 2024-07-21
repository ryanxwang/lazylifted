use crate::search::{HeuristicValue, Transition};
use ordered_float::Float;
use std::sync::atomic::AtomicUsize;

/// [`NodeId`] are used to uniquely identify nodes in the search space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    #[inline(always)]
    pub fn id(&self) -> usize {
        self.0
    }
}

/// [`NO_NODE`] is a special state id that should only be used to indicate that
/// a node has no parent. We use this instead of an [`Option<NodeId>`] to avoid
/// the overhead of an [`Option`] type.
pub const NO_NODE: NodeId = NodeId(usize::MAX);

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
    node_id: NodeId,
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
    parent_id: NodeId,
}

impl<T> SearchNode<T>
where
    T: Transition,
{
    pub fn update_parent(&mut self, parent_id: NodeId, transition: T) {
        self.parent_id = parent_id;
        self.transition = transition;
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
        assert_eq!(
            self.status,
            SearchNodeStatus::Open,
            "Node must be open to close it"
        );
        self.status = SearchNodeStatus::Closed;
    }

    pub fn get_status(&self) -> SearchNodeStatus {
        self.status
    }

    pub fn get_node_id(&self) -> NodeId {
        self.node_id
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

    pub fn get_parent_id(&self) -> NodeId {
        self.parent_id
    }

    pub fn get_transition(&self) -> &T {
        &self.transition
    }
}

/// This generator is used to create unique node ids.
#[derive(Debug)]
struct NodeIdGenerator {
    counter: AtomicUsize,
}

impl NodeIdGenerator {
    fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }

    fn next_node_id(&self) -> NodeId {
        NodeId(
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}

#[derive(Debug)]
pub struct SearchNodeFactory {
    id_generator: NodeIdGenerator,
}

impl SearchNodeFactory {
    pub fn new() -> Self {
        Self {
            id_generator: NodeIdGenerator::new(),
        }
    }

    pub fn new_node<T>(&mut self, parent_id: NodeId, transition: T) -> SearchNode<T>
    where
        T: Transition,
    {
        SearchNode {
            node_id: self.id_generator.next_node_id(),
            status: SearchNodeStatus::New,
            f: HeuristicValue::infinity(),
            g: HeuristicValue::infinity(),
            h: HeuristicValue::infinity(),
            transition,
            parent_id,
        }
    }

    pub fn new_root_node<T>(&mut self) -> SearchNode<T>
    where
        T: Transition,
    {
        SearchNode {
            node_id: self.id_generator.next_node_id(),
            status: SearchNodeStatus::New,
            f: HeuristicValue::infinity(),
            g: HeuristicValue::infinity(),
            h: HeuristicValue::infinity(),
            transition: T::no_transition(),
            parent_id: NO_NODE,
        }
    }
}
