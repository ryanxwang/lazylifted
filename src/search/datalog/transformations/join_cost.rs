use std::{collections::HashSet, hash::Hash};

use crate::search::{
    atom,
    datalog::{arguments::Arguments, atom::Atom, rules::Rule, term::Term},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinCostType {
    FastDownward,
    Helmert09,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JoinCost((isize, isize, isize));

impl JoinCost {
    pub fn new(n: isize, max: isize, min: isize, join_cost_type: JoinCostType) -> Self {
        match join_cost_type {
            JoinCostType::FastDownward => Self((min - n, max - n, -n)),
            JoinCostType::Helmert09 => Self((n - max, n - min, n)),
        }
    }
}

fn joining_variables(rule: &Rule, atom1: &Atom, atom2: &Atom) -> HashSet<usize> {
    let variables_union: HashSet<usize> = atom1
        .variables_set()
        .union(&atom2.variables_set())
        .cloned()
        .collect();

    let mut variables_elsewhere_in_rule = HashSet::new();
    variables_elsewhere_in_rule.extend(rule.effect().variables_set());
    for condition in rule.conditions() {
        if condition == atom1 || condition == atom2 {
            continue;
        }
        variables_elsewhere_in_rule.extend(condition.variables_set());
    }

    variables_union
        .intersection(&variables_elsewhere_in_rule)
        .cloned()
        .collect()
}

impl JoinCostType {
    pub fn calculate_join_cost(&self, rule: &Rule, atom1: &Atom, atom2: &Atom) -> JoinCost {
        let free_variables_atom1 = atom1.variables_set();
        let free_variables_atom2 = atom2.variables_set();

        let arity1 = free_variables_atom1.len();
        let arity2 = free_variables_atom2.len();

        let max = std::cmp::max(arity1, arity2);
        let min = std::cmp::min(arity1, arity2);
        let n = match self {
            JoinCostType::FastDownward => free_variables_atom1
                .intersection(&free_variables_atom2)
                .count(),
            JoinCostType::Helmert09 => joining_variables(rule, atom1, atom2).len(),
        };

        JoinCost::new(n as isize, max as isize, min as isize, *self)
    }
}
