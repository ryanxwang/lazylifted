use crate::search::datalog::{
    arguments::Arguments, atom::Atom, program::Program, rules::RuleTrait,
};

/// Add a special epsilon predicate to the conditions of all action
/// applicability rules. This should be applied before removing action
/// predicates.
pub fn restrict_immediate_applicability(mut program: Program) -> Program {
    let epsilon_predicate = program.new_auxillary_predicate(Some("epsilon".to_string()));

    let epsilon = Atom::new(Arguments::new(vec![]), epsilon_predicate, true);

    for rule in program
        .rules
        .iter_mut()
        .filter(|rule| rule.effect().is_artificial_predicate())
    {
        rule.conditions_mut().push(epsilon.clone());
    }

    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::{
        datalog::{program::Program, Annotation, AnnotationGenerator},
        Task,
    };
    use crate::test_utils::*;
    use itertools::Itertools;
    use std::rc::Rc;

    #[test]
    fn test_restrict_immediate_applicability() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = restrict_immediate_applicability(program);

        assert_eq!(
            program.predicate_names,
            vec![
                "clear",
                "on-table",
                "arm-empty",
                "holding",
                "on",
                "applicable-pickup",
                "applicable-putdown",
                "applicable-stack",
                "applicable-unstack",
                "@epsilon"
            ]
        );
        assert_eq!(
            program
                .rules
                .iter()
                .map(|rule| format!("{}", rule))
                .collect_vec(),
            vec![
                // pickup applicability rule
                "(5(?0) <- 2(), 1(?0), 0(?0), 9()  | weight: 1; annotation: None; schema_index: 0)",
                // pickup effect rules, only one add effect (holding ?ob)
                "(3(?0) <- 5(?0)  | weight: 0; annotation: None; schema_index: 0)",
                // putdown applicability rule
                "(6(?0) <- 3(?0), 9()  | weight: 1; annotation: None; schema_index: 1)",
                // putdown effect rules, add effects (clear ?ob), (arm-empty), (on-table ?ob)
                "(0(?0) <- 6(?0)  | weight: 0; annotation: None; schema_index: 1)",
                "(2() <- 6(?0)  | weight: 0; annotation: None; schema_index: 1)",
                "(1(?0) <- 6(?0)  | weight: 0; annotation: None; schema_index: 1)",
                // stack applicability rule
                "(7(?0, ?1) <- 3(?0), 0(?1), 9()  | weight: 1; annotation: None; schema_index: 2)",
                // stack effect rules, add effects (arm-empty) (clear ?ob) (on ?ob ?underob)
                "(2() <- 7(?0, ?1)  | weight: 0; annotation: None; schema_index: 2)",
                "(0(?0) <- 7(?0, ?1)  | weight: 0; annotation: None; schema_index: 2)",
                "(4(?0, ?1) <- 7(?0, ?1)  | weight: 0; annotation: None; schema_index: 2)",
                // unstack applicability rule
                "(8(?0, ?1) <- 2(), 0(?0), 4(?0, ?1), 9()  | weight: 1; annotation: None; schema_index: 3)",
                // unstack effect rules, add effects (holding ?ob) (clear ?underob)
                "(3(?0) <- 8(?0, ?1)  | weight: 0; annotation: None; schema_index: 3)",
                "(0(?1) <- 8(?0, ?1)  | weight: 0; annotation: None; schema_index: 3)"
            ]
        );
    }
}
