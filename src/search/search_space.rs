use crate::search::{
    states::SchemaOrInstantiation, Action, NodeId, PartialActionDiff, Plan, SearchNode,
    SearchNodeFactory, Transition, NO_NODE,
};
use segvec::{Linear, SegVec};
use std::{
    collections::HashMap,
    hash::{BuildHasher, Hash, RandomState},
};

/// A [`SearchSpace`] is a data structure for managing the states and nodes
/// during a search. Here state and state transitions are both abstract. That
/// is, we only require the state to be hashable and the transition to satisfy
/// the [`Transition`] trait.
#[derive(Debug)]
pub struct SearchSpace<S: Hash, T: Transition> {
    root_node_id: NodeId,
    nodes: SegVec<SearchNode<T>, Linear>,
    states: SegVec<S, Linear>,
    registered_nodes: HashMap<u64, NodeId>,
    state_build_hasher: RandomState,
    search_node_factory: SearchNodeFactory,
}

impl<S: Hash, T: Transition> SearchSpace<S, T> {
    pub fn new(initial_state: S) -> Self {
        let state_build_hasher = RandomState::new();

        let mut nodes = SegVec::new();
        let mut states = SegVec::new();
        let mut registered_states = HashMap::new();
        let mut search_node_factory = SearchNodeFactory::new();

        let root_node = search_node_factory.new_root_node();
        let root_node_id = root_node.get_node_id();
        print!("Root state id: {:?}", root_node_id);
        registered_states.insert(state_build_hasher.hash_one(&initial_state), root_node_id);
        nodes.push(root_node);
        states.push(initial_state);

        Self {
            root_node_id,
            nodes,
            states,
            registered_nodes: registered_states,
            state_build_hasher,
            search_node_factory,
        }
    }

    pub fn insert_or_get_node(
        &mut self,
        state: S,
        transition: T,
        parent_id: NodeId,
    ) -> &mut SearchNode<T> {
        let state_hash = self.state_build_hasher.hash_one(&state);
        match self.registered_nodes.get(&state_hash) {
            Some(&node_id) => {
                return self.get_node_mut(node_id);
            }
            None => {
                self.states.push(state);
                let new_node = self.search_node_factory.new_node(parent_id, transition);
                let node_id = new_node.get_node_id();
                print!("New state id: {:?}", node_id);
                self.nodes.push(new_node);
                self.registered_nodes.insert(state_hash, node_id);
                return self.get_node_mut(node_id);
            }
        }
    }

    #[inline(always)]
    pub fn get_root_node(&self) -> &SearchNode<T> {
        self.get_node(self.root_node_id)
    }

    #[inline(always)]
    pub fn get_root_node_mut(&mut self) -> &mut SearchNode<T> {
        self.get_node_mut(self.root_node_id)
    }

    #[inline(always)]
    pub fn get_node(&self, node_id: NodeId) -> &SearchNode<T> {
        self.nodes.get(node_id.id()).expect("Invalid state id")
    }

    #[inline(always)]
    pub fn get_node_mut(&mut self, node_id: NodeId) -> &mut SearchNode<T> {
        print!("State id: {:?}", node_id);
        self.nodes.get_mut(node_id.id()).expect("Invalid state id")
    }

    #[inline(always)]
    pub fn get_state(&self, node_id: NodeId) -> &S {
        self.states.get(node_id.id()).expect("Invalid state id")
    }
}

impl<S: Hash> SearchSpace<S, Action> {
    pub fn extract_plan(&self, goal_node: &SearchNode<Action>) -> Plan {
        let mut steps = vec![];
        let mut current_node = goal_node;
        while NO_NODE != current_node.get_parent_id() {
            steps.push(current_node.get_transition().clone());
            current_node = self.get_node(current_node.get_parent_id());
        }
        steps.reverse();
        Plan::new(steps)
    }
}

impl<S: Hash> SearchSpace<S, PartialActionDiff> {
    pub fn extract_plan(&self, goal_node: &SearchNode<PartialActionDiff>) -> Plan {
        let mut steps = vec![];
        let mut current_node = goal_node;

        while NO_NODE != current_node.get_parent_id() {
            let mut instantiations = vec![];
            while let PartialActionDiff::Instantiation(object_index) = current_node.get_transition()
            {
                instantiations.push(*object_index);
                current_node = self.get_node(current_node.get_parent_id());
            }
            instantiations.reverse();

            match current_node.get_transition() {
                PartialActionDiff::Schema(schema_index) => {
                    let action = Action::new(*schema_index, instantiations);
                    steps.push(action);
                }
                _ => panic!("Invalid transition type"),
            }
            current_node = self.get_node(current_node.get_parent_id());
        }
        steps.reverse();
        Plan::new(steps)
    }
}

impl<S: Hash> SearchSpace<S, SchemaOrInstantiation> {
    pub fn extract_plan(&self, goal_node: &SearchNode<SchemaOrInstantiation>) -> Plan {
        let mut steps = vec![];
        let mut current_node = goal_node;

        while NO_NODE != current_node.get_parent_id() {
            match current_node.get_transition() {
                SchemaOrInstantiation::Instantiation(action) => {
                    steps.push(action.clone());
                }
                SchemaOrInstantiation::Schema(_schema_index) => {}
            }
            current_node = self.get_node(current_node.get_parent_id());
        }

        steps.reverse();
        Plan::new(steps)
    }
}
