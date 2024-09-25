use crate::search::datalog::{
    program::Program,
    rules::{GenericRule, Rule},
};

pub fn remove_action_predicates(program: Program) -> Program {
    let mut new_rules = vec![];

    for applicability_rule in program
        .rules
        .iter()
        // only the applicability rules have articificial predicates
        .filter(|rule| rule.effect().is_artificial_predicate())
    {
        let applicability_predicate = applicability_rule.effect().predicate_index();
        for effect_rule in program.rules.iter().filter(|rule| {
            rule.conditions().len() == 1
                && rule.conditions()[0].predicate_index() == applicability_predicate
        }) {
            new_rules.push(Rule::new_generic(GenericRule::new(
                effect_rule.effect().clone(),
                applicability_rule.conditions().to_owned(),
                applicability_rule.weight() + effect_rule.weight(),
                applicability_rule.annotation().clone(),
                applicability_rule.schema_index().unwrap(),
            )))
        }
    }

    Program {
        rules: new_rules,
        ..program
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::{
            datalog::{program::Program, Annotation, AnnotationGenerator},
            Task,
        },
        test_utils::*,
    };
    use itertools::Itertools;
    use std::rc::Rc;

    #[test]
    fn test_remove_action_predicates() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = remove_action_predicates(program);

        // the action predicates are still recorded, even if not used in the
        // rules
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
                "applicable-unstack"
            ]
        );
        assert_eq!(
            program
                .rules
                .iter()
                .map(|rule| format!("{}", rule))
                .collect_vec(),
            vec![
                // pickup rules, only one add effect (holding ?ob)
                "(3(?0) <- 2(), 1(?0), 0(?0)  | weight: 1; annotation: None; schema_index: 0)",
                // putdown rules, add effects (clear ?ob), (arm-empty), (on-table ?ob)
                "(0(?0) <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                "(2() <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                "(1(?0) <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                // stack rules, add effects (arm-empty) (clear ?ob) (on ?ob ?underob)
                "(2() <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                "(0(?0) <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                "(4(?0, ?1) <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                // unstack applicability rule, add effects (holding ?ob) (clear ?underob)
                "(3(?0) <- 2(), 0(?0), 4(?0, ?1)  | weight: 1; annotation: None; schema_index: 3)",
                "(0(?1) <- 2(), 0(?0), 4(?0, ?1)  | weight: 1; annotation: None; schema_index: 3)"
            ]
        );
    }
}
