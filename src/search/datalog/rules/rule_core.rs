use crate::search::datalog::atom::Atom;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RuleCore {
    effect: Atom,
    conditions: Vec<Atom>,
}
