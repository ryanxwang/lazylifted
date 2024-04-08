use crate::search::{search_engines::SearchNode, states::SparsePackedState, Action};
use segvec::{Linear, SegVec};
use std::{
    collections::HashMap,
    hash::{BuildHasher, RandomState},
    sync::atomic::AtomicUsize,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(usize);

impl StateId {
    /// Create a new state id, starting from 0. Each call to this function will
    /// return a new unique id.
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        Self(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

pub const NO_STATE: StateId = StateId(usize::MAX);

pub struct SearchSpace {
    root_state_id: StateId,
    nodes: SegVec<SearchNode, Linear>,
    states: SegVec<SparsePackedState, Linear>,
    registered_states: HashMap<u64, StateId>,
    state_build_hasher: RandomState,
}

impl SearchSpace {
    pub fn new(initial_state: SparsePackedState) -> Self {
        let state_build_hasher = RandomState::new();

        let mut nodes = SegVec::new();
        let mut states = SegVec::new();
        let mut registered_states = HashMap::new();

        let root_node = SearchNode::new_without_parent();
        let root_state_id = root_node.get_state_id();
        registered_states.insert(state_build_hasher.hash_one(&initial_state), root_state_id);
        nodes.push(root_node);
        states.push(initial_state);

        Self {
            root_state_id,
            nodes,
            states,
            registered_states,
            state_build_hasher,
        }
    }

    pub fn insert_or_get_node(
        &mut self,
        state: SparsePackedState,
        action: Action,
        parent_id: StateId,
    ) -> &mut SearchNode {
        let state_hash = self.state_build_hasher.hash_one(&state);
        match self.registered_states.get(&state_hash) {
            Some(&state_id) => {
                return self.get_node_mut(state_id);
            }
            None => {
                self.states.push(state);
                let new_node = SearchNode::new_with_parent(parent_id, action);
                let state_id = new_node.get_state_id();
                self.nodes.push(new_node);
                self.registered_states.insert(state_hash, state_id);
                return self.get_node_mut(state_id);
            }
        }
    }

    pub fn extract_plan(&self, goal_node: &SearchNode) -> Vec<Action> {
        let mut plan = vec![];
        let mut current_node = goal_node;
        while NO_STATE != current_node.get_parent_id() {
            plan.push(current_node.get_action().clone());
            current_node = self.get_node(current_node.get_parent_id());
        }
        plan.reverse();
        plan
    }

    pub fn get_root_node_mut(&mut self) -> &mut SearchNode {
        self.get_node_mut(self.root_state_id)
    }

    pub fn get_node(&self, state_id: StateId) -> &SearchNode {
        self.nodes.get(state_id.0).expect("Invalid state id")
    }

    pub fn get_node_mut(&mut self, state_id: StateId) -> &mut SearchNode {
        self.nodes.get_mut(state_id.0).expect("Invalid state id")
    }

    pub fn get_state(&self, state_id: StateId) -> &SparsePackedState {
        self.states.get(state_id.0).expect("Invalid state id")
    }

    pub fn len(&self) -> usize {
        self.registered_states.len()
    }
}
