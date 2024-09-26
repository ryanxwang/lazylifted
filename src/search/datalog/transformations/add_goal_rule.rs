use std::rc::Rc;

use crate::search::{
    datalog::{
        arguments::Arguments,
        atom::Atom,
        program::Program,
        rules::{ProductRule, Rule},
        term::Term,
        AnnotationGenerator, RuleCategory,
    },
    goal, Task,
};

pub fn add_goal_rule(
    mut program: Program,
    task: Rc<Task>,
    annotation_generator: &AnnotationGenerator,
) -> Program {
    let goal_id = program.new_auxillary_predicate(Some("goal".to_string()));
    program.goal_predicate_index = Some(goal_id);
    let goal = Atom::new(Arguments::new(vec![]), goal_id, true);

    let annotation = annotation_generator(RuleCategory::Goal, task.clone());

    let conditions: Vec<Atom> = task
        .goal
        .atoms()
        .iter()
        .map(|atom| {
            assert!(atom.is_positive());
            let terms = atom
                .arguments()
                .iter()
                .map(|object_index| Term::new_object(*object_index))
                .collect();

            Atom::new(Arguments::new(terms), atom.predicate_index(), false)
        })
        .collect();

    let goal_rule = Rule::new_product(ProductRule::new(goal, conditions, 0.0, annotation));
    program.rules.push(goal_rule);

    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::datalog::{
            transformations::{convert_rules_to_normal_form, remove_action_predicates},
            Annotation,
        },
        test_utils::*,
    };
    use std::rc::Rc;

    #[test]
    fn test_blocksworld_goal_rule() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = remove_action_predicates(program);
        program = convert_rules_to_normal_form(program);
        program = add_goal_rule(program, task, &annotation_generator);

        assert_eq!(program.predicate_names.len(), 13);
        assert_eq!(program.predicate_names.last().unwrap(), "@goal");
        assert_eq!(
            program.rules.last().unwrap().to_string(),
            "(12() <- 0(3), 4(3, 1), 4(1, 2), 4(2, 0), 1(0)  | weight: 0; annotation: None)"
        );
    }

    #[test]
    fn test_spanner_goal_rule() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = remove_action_predicates(program);
        program = convert_rules_to_normal_form(program);
        program = add_goal_rule(program, task, &annotation_generator);

        assert_eq!(program.predicate_names.len(), 13);
        assert_eq!(program.predicate_names.last().unwrap(), "@goal");
        assert_eq!(
            program.rules.last().unwrap().to_string(),
            "(12() <- 4(5), 4(6)  | weight: 0; annotation: None)"
        );
    }
}
