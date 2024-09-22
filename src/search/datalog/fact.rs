use crate::search::datalog::atom::Atom;
use global_counter::global_counter;

pub type FactId = usize;
global_counter!(FACT_COUNTER, FactId, 0);

#[derive(Debug, Clone)]
pub struct Fact {
    // Fact IDs are used to identify facts such that data structures indexed by
    // facts don't need to be hash maps. It starts off as None and is set as
    // used.
    id: Option<FactId>,
    atom: Atom,
    cost: f64,
}

impl Fact {
    pub fn new(atom: Atom, cost: f64) -> Self {
        Self {
            id: None,
            atom,
            cost,
        }
    }

    pub fn id(&self) -> Option<FactId> {
        self.id
    }

    pub fn set_id(&mut self) {
        self.id = Some(FACT_COUNTER.get_cloned());
        FACT_COUNTER.inc();
    }

    pub fn atom(&self) -> &Atom {
        &self.atom
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    pub fn reset_id_counter() {
        FACT_COUNTER.reset();
    }
}

impl PartialEq for Fact {
    fn eq(&self, other: &Self) -> bool {
        self.atom == other.atom
    }
}

impl Eq for Fact {}
