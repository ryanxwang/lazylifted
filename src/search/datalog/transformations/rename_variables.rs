use crate::search::datalog::{
    arguments::Arguments, program::Program, rules::RuleTrait, term::Term,
};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};

pub fn rename_variables(mut program: Program) -> Program {
    for rule in program.rules.iter_mut() {
        let mut seen_variables: HashSet<usize> = HashSet::new();
        let mut old_to_new_variable_index: HashMap<usize, usize> = HashMap::new();

        // all the variables should appear in the conditions
        for condition in rule.conditions_mut() {
            let mut new_terms = vec![];

            for term in condition.arguments() {
                match term {
                    Term::Object(_) => {
                        new_terms.push(*term);
                    }
                    Term::Variable {
                        variable_index,
                        type_index,
                    } => {
                        if !seen_variables.contains(variable_index) {
                            let new_index = seen_variables.len();
                            seen_variables.insert(*variable_index);
                            old_to_new_variable_index.insert(*variable_index, new_index);
                        }
                        new_terms.push(Term::Variable {
                            variable_index: old_to_new_variable_index[variable_index],
                            type_index: *type_index,
                        })
                    }
                }
            }

            *condition = condition
                .clone()
                .with_arguments(Arguments::new(SmallVec::from_vec(new_terms)));
        }

        // update the effect
        let mut new_terms = vec![];
        for term in rule.effect().arguments() {
            match term {
                Term::Object(_) => {
                    new_terms.push(*term);
                }
                Term::Variable {
                    variable_index,
                    type_index,
                } => new_terms.push(Term::Variable {
                    variable_index: old_to_new_variable_index[variable_index],
                    type_index: *type_index,
                }),
            }
        }

        let effect = rule.effect_mut();
        *effect = effect
            .clone()
            .with_arguments(Arguments::new(SmallVec::from_vec(new_terms)));
        rule.variable_source_mut()
            .update_variable_indices(&old_to_new_variable_index);
        rule.update_variable_position_in_effect();
    }

    program
}
