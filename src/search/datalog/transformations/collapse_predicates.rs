use std::collections::HashSet;

use crate::search::datalog::{program::Program, rules::RuleTrait};

pub fn collapse_predicates(mut program: Program) -> (bool, Program) {
    let mut to_delete: HashSet<usize> = HashSet::new();

    for rule_index in 0..program.rules.len() {
        // only consider deleting rules that introduce artificial predicates,
        // there should only be one rule introducing each artificial predicate,
        // which simplify the collapsing process
        if !program.rules[rule_index].effect().is_artificial_predicate()
            || to_delete.contains(&rule_index)
        {
            continue;
        }

        let mut equivalent_predicates: HashSet<usize> = HashSet::new();

        // find all the equivalent rules
        for other_rule_index in (rule_index + 1)..program.rules.len() {
            if !program.rules[other_rule_index]
                .effect()
                .is_artificial_predicate()
                || to_delete.contains(&other_rule_index)
            {
                continue;
            }

            if program.rules[rule_index].equivalent_to(&program.rules[other_rule_index]) {
                to_delete.insert(other_rule_index);
                equivalent_predicates
                    .insert(program.rules[other_rule_index].effect().predicate_index());
            }
        }

        let retained_predicate = program.rules[rule_index].effect().predicate_index();

        // update the conditions of the remaining rules
        for other_rule_index in 0..program.rules.len() {
            if !to_delete.contains(&other_rule_index) {
                continue;
            }

            for condition_index in 0..program.rules[other_rule_index].conditions().len() {
                if !equivalent_predicates.contains(
                    &program.rules[other_rule_index].conditions()[condition_index]
                        .predicate_index(),
                ) {
                    continue;
                }

                program.rules[other_rule_index]
                    .update_predicate_index(retained_predicate, condition_index);
            }
        }
    }

    // actually remove the rules
    program.rules = program
        .rules
        .into_iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            if to_delete.contains(&index) {
                None
            } else {
                Some(rule)
            }
        })
        .collect();

    (!to_delete.is_empty(), program)
}
