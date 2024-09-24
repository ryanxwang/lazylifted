use crate::search::{
    datalog::{
        arguments::Arguments,
        atom::Atom,
        program::Program,
        rules::{ProjectRule, Rule},
        term::Term,
        transformations::connected_components::split_into_connected_components,
        Annotation,
    },
    Task,
};

/// Use projection rules to remove constant arguments from the rules.
/// Specifically, given a rule with more than one condition, for each condition
/// with a constant, e.g. `p(a, ?b)`, we add a projection rule
/// ```Datalog
///     aux(?b) <- p(a, ?b)
/// ```
/// and replace the condition `p(a, ?b)` in the original rule with `aux(?b)`.
///
/// This came from the original Powerlifted code. A copy of the original comment
/// is below:
///
/// > This is a quick solution to the problem with constant in join rules.
/// > Ideally we want to either check before the join if the constants match, or
/// > make the rule matcher also take into account the constants.
fn project_away_constant_arguments(mut program: Program) -> Program {
    // TODO-someday: Test this
    let mut new_rules = vec![];
    for rule_index in 0..program.rules.len() {
        for condition_index in 0..program.rules[rule_index].conditions().len() {
            let condition = &program.rules[rule_index].conditions()[condition_index];
            let requires_projection = condition
                .arguments()
                .iter()
                .any(|argument| argument.is_object());
            if !requires_projection {
                continue;
            }
            // we will need to clone anyway, doing it here avoids borrowing
            // issues
            let condition = condition.clone();

            let free_variables: Vec<Term> = condition
                .arguments()
                .iter()
                .filter(|argument| argument.is_variable())
                .cloned()
                .collect();

            let aux_predicate_index = program.new_auxillary_predicate();
            let aux_atom = Atom::new(Arguments::new(free_variables), aux_predicate_index, true);
            let project_rule = Rule::new_project(ProjectRule::new(
                aux_atom.clone(),
                condition,
                0.0,
                Annotation::None,
            ));
            new_rules.push(project_rule);
            program.rules[rule_index].update_single_condition(aux_atom, condition_index);
        }
    }

    program.rules.extend(new_rules);
    program
}

pub fn convert_rules_to_normal_form(mut program: Program) -> Program {
    for i in 0..program.rules.len() {
        program = split_into_connected_components(program, i);
    }

    program = project_away_constant_arguments(program);

    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::datalog::{transformations::remove_action_predicates, AnnotationGenerator},
        test_utils::*,
    };
    use std::rc::Rc;

    #[test]
    fn project_away_constant_arguments_does_nothing_in_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task, annotation_generator);
        let original_program = program.clone();
        program = project_away_constant_arguments(program);

        assert_eq!(program, original_program);
    }
}
