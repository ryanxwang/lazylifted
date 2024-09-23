use std::{collections::HashMap, rc::Rc};

use itertools::Itertools;
use tracing_subscriber::fmt::format;

use crate::search::{
    datalog::{
        atom::Atom,
        fact::Fact,
        rules::{GenericRule, Rule},
        transformations::{remove_action_predicates, TransformationOptions},
        AnnotationGenerator, RuleCategory,
    },
    ActionSchema, Task,
};

/// If true, the program will panic if a negative precondition is encountered.
/// Otherwise, the precondition will be ignored, which is equivalent to an
/// additional relaxation of the planning problem on top of delete relaxation.
const PANIC_ON_NEGATIVE_PRECONDITIONS: bool = false;

#[derive(Debug)]
pub struct Program {
    pub(super) rules: Vec<Rule>,
    pub(super) task: Rc<Task>,
    // Predicate names for the atoms, including ones generated when building the
    // program.
    pub(super) predicate_names: Vec<String>,
    pub(super) predicate_name_to_index: HashMap<String, usize>,
}

impl Program {
    pub fn new_with_transformations(
        task: Rc<Task>,
        annotation_generator: AnnotationGenerator,
        transformation_options: &TransformationOptions,
    ) -> Self {
        let mut program = Self::new(task.clone(), annotation_generator);

        if transformation_options.remove_action_predicates {
            program = remove_action_predicates(program);
        }

        program
    }

    /// Generate a program for the given task. This is intentionally not public
    /// because users should use [`Self::new_with_transformations`] instead.
    fn new(task: Rc<Task>, annotation_generator: AnnotationGenerator) -> Self {
        let mut predicate_names: Vec<String> = task
            .predicates
            .iter()
            .map(|p| p.name.clone().to_string())
            .collect();
        let mut predicate_name_to_index = predicate_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        let mut rules = vec![];
        for action_schema in task.action_schemas() {
            rules.push(Self::generate_action_applicability_rule(
                action_schema,
                &mut predicate_names,
                &mut predicate_name_to_index,
                &annotation_generator,
                task.clone(),
            ));

            rules.append(&mut Self::generate_action_effect_rules(
                action_schema,
                &mut predicate_name_to_index,
                &annotation_generator,
                task.clone(),
            ));
        }

        Self {
            rules,
            task,
            predicate_names,
            predicate_name_to_index,
        }
    }

    /// Generate the action applicability rule, where we create a new predicate
    /// `applicable-a` for each action schema and add a rule `applicable-a <-
    /// pre(a)` with weight being the action cost.
    fn generate_action_applicability_rule(
        action_schema: &ActionSchema,
        predicate_names: &mut Vec<String>,
        predicate_name_to_index: &mut HashMap<String, usize>,
        annotation_generator: &AnnotationGenerator,
        task: Rc<Task>,
    ) -> Rule {
        let predicate_index = predicate_names.len();
        assert!(
            !predicate_name_to_index
                .contains_key(&Self::applicability_predicate_name(action_schema)),
            "Predicate name {} already exists",
            Self::applicability_predicate_name(action_schema)
        );
        predicate_name_to_index.insert(
            Self::applicability_predicate_name(action_schema),
            predicate_index,
        );
        predicate_names.push(Self::applicability_predicate_name(action_schema));
        let effect = Atom::new_from_action_schema(action_schema, predicate_index);

        let conditions = action_schema
            .preconditions()
            .iter()
            .filter_map(|p| {
                if p.is_negated() {
                    if PANIC_ON_NEGATIVE_PRECONDITIONS {
                        panic!("Negated preconditions are not supported for Datalog");
                    } else {
                        None
                    }
                } else {
                    Some(Atom::new_from_atom_schema(p.underlying()))
                }
            })
            // According to comments in Powerlifted, this has an effect in
            // the performance for some domains
            .rev()
            .collect_vec();
        let annotation = annotation_generator(
            RuleCategory::ActionApplicability {
                schema_index: action_schema.index(),
            },
            task.clone(),
        );

        Rule::new_generic(GenericRule::new(
            effect,
            conditions,
            1.0,
            annotation,
            action_schema.index(),
        ))
    }

    /// Generate the action effect rules, where we create rules of the form `p
    /// <- applicable-a` for each p in add(a)
    fn generate_action_effect_rules(
        action_schema: &ActionSchema,
        predicate_name_to_index: &mut HashMap<String, usize>,
        annotation_generator: &AnnotationGenerator,
        task: Rc<Task>,
    ) -> Vec<Rule> {
        let conditions = vec![Atom::new_from_action_schema(
            action_schema,
            predicate_name_to_index[&Self::applicability_predicate_name(action_schema)],
        )];

        action_schema
            .effects()
            .iter()
            .filter_map(|e| {
                if e.is_negated() {
                    return None;
                }

                let effect = Atom::new_from_atom_schema(e.underlying());
                let annotation = annotation_generator(RuleCategory::ActionEffect, task.clone());

                Some(Rule::new_generic(GenericRule::new(
                    effect,
                    conditions.clone(),
                    0.0,
                    annotation,
                    action_schema.index(),
                )))
            })
            .collect()
    }

    fn applicability_predicate_name(action_schema: &ActionSchema) -> String {
        format!("applicable-{}", action_schema.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::datalog::Annotation;
    use crate::search::Task;
    use crate::test_utils::*;

    #[test]
    fn test_new_program_without_transformations() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);
        let transformation_options = TransformationOptions::new(false, false, false);

        let program = Program::new_with_transformations(
            task.clone(),
            annotation_generator,
            &transformation_options,
        );

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
                // pickup applicability rule
                "(5(?0) <- 2(), 1(?0), 0(?0)  | weight: 1; annotation: None; schema_index: 0)",
                // pickup effect rules, only one add effect (holding ?ob)
                "(3(?0) <- 5(?0)  | weight: 0; annotation: None; schema_index: 0)",
                // putdown applicability rule
                "(6(?0) <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                // putdown effect rules, add effects (clear ?ob), (arm-empty), (on-table ?ob)
                "(0(?0) <- 6(?0)  | weight: 0; annotation: None; schema_index: 1)",
                "(2() <- 6(?0)  | weight: 0; annotation: None; schema_index: 1)",
                "(1(?0) <- 6(?0)  | weight: 0; annotation: None; schema_index: 1)",
                // stack applicability rule
                "(7(?0, ?1) <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                // stack effect rules, add effects (arm-empty) (clear ?ob) (on ?ob ?underob)
                "(2() <- 7(?0, ?1)  | weight: 0; annotation: None; schema_index: 2)",
                "(0(?0) <- 7(?0, ?1)  | weight: 0; annotation: None; schema_index: 2)",
                "(4(?0, ?1) <- 7(?0, ?1)  | weight: 0; annotation: None; schema_index: 2)",
                // unstack applicability rule
                "(8(?0, ?1) <- 2(), 0(?0), 4(?0, ?1)  | weight: 1; annotation: None; schema_index: 3)",
                // unstack effect rules, add effects (holding ?ob) (clear ?underob)
                "(3(?0) <- 8(?0, ?1)  | weight: 0; annotation: None; schema_index: 3)",
                "(0(?1) <- 8(?0, ?1)  | weight: 0; annotation: None; schema_index: 3)"
            ]
        );
    }

    #[test]
    fn test_new_program_with_action_predicate_removal() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);
        let transformation_options = TransformationOptions::new(false, false, true);

        let program = Program::new_with_transformations(
            task.clone(),
            annotation_generator,
            &transformation_options,
        );

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
