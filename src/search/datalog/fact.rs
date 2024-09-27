use std::{
    collections::HashMap,
    fmt::Display,
    hash::{BuildHasher, Hash, RandomState},
};

use crate::search::{
    datalog::{arguments::Arguments, atom::Atom, term::Term},
    DBState, Task,
};
use ordered_float::OrderedFloat;
use segvec::{Linear, SegVec};

pub type FactCost = OrderedFloat<f64>;

#[derive(Debug, Clone)]
pub struct Fact {
    atom: Atom,
    cost: FactCost,
}

impl Fact {
    pub fn new(atom: Atom, cost: FactCost) -> Self {
        assert!(atom.arguments().iter().all(|term| term.is_object()));
        Self {
            atom,
            cost: FactCost::from(cost),
        }
    }

    pub fn atom(&self) -> &Atom {
        &self.atom
    }

    pub fn cost(&self) -> FactCost {
        self.cost
    }
}

impl PartialEq for Fact {
    fn eq(&self, other: &Self) -> bool {
        self.atom == other.atom
    }
}

impl Eq for Fact {}

impl Display for Fact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(fact {}, cost {})", self.atom, self.cost)
    }
}

impl Hash for Fact {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.atom.hash(state);
    }
}

// Get the state specific facts from the state, will ignore the static
// predicates
pub fn facts_from_state(state: &DBState, task: &Task) -> Vec<Fact> {
    let mut facts = vec![];
    for atom in state.atoms() {
        if task.static_predicates().contains(&atom.predicate_index()) {
            continue;
        }

        let terms: Vec<Term> = atom
            .arguments()
            .iter()
            .map(|&object_index| Term::new_object(object_index))
            .collect();
        facts.push(Fact::new(
            Atom::new(Arguments::new(terms), atom.predicate_index(), false),
            FactCost::from(0.0),
        ));
    }

    facts
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactId(usize);

#[derive(Debug)]
pub struct FactRegistry {
    facts: SegVec<Fact, Linear>,
    reached_facts: HashMap<u64, FactId>,
    hasher: RandomState,
}

impl FactRegistry {
    pub fn new() -> Self {
        Self {
            facts: SegVec::new(),
            reached_facts: HashMap::new(),
            hasher: RandomState::new(),
        }
    }

    pub fn get_id(&self, fact: &Fact) -> Option<FactId> {
        let fact_hash = self.hasher.hash_one(fact);
        self.reached_facts.get(&fact_hash).copied()
    }

    pub fn replace_at_id(&mut self, fact_id: FactId, fact: Fact) {
        self.facts[fact_id.0] = fact;
    }

    pub fn add_or_get_fact(&mut self, fact: Fact) -> FactId {
        let fact_hash = self.hasher.hash_one(&fact);
        match self.reached_facts.get(&fact_hash) {
            Some(&fact_id) => fact_id,
            None => {
                let fact_id = FactId(self.facts.len());
                self.facts.push(fact);
                self.reached_facts.insert(fact_hash, fact_id);
                fact_id
            }
        }
    }

    pub fn get_by_id(&self, fact_id: FactId) -> &Fact {
        &self.facts[fact_id.0]
    }
}
