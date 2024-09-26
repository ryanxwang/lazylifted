use std::{collections::HashMap, rc::Rc};

use itertools::Itertools;
use tracing_subscriber::fmt::format;

use crate::search::{
    datalog::{
        atom::Atom,
        fact::Fact,
        rules::{GenericRule, Rule, RuleIndex, RuleTrait},
        transformations::{
            add_goal_rule, convert_rules_to_normal_form, generate_static_facts,
            remove_action_predicates, TransformationOptions,
        },
        AnnotationGenerator, RuleCategory,
    },
    ActionSchema, Task,
};

#[derive(Debug, Clone)]
pub struct Program {
    // Don't forget to update the PartialEq implementation when adding new
    // fields.
    pub(super) static_facts: Vec<Fact>,
    pub(super) rules: Vec<Rule>,
    pub(super) task: Rc<Task>,
    // Predicate names for the atoms, including ones generated when building the
    // program.
    pub(super) predicate_names: Vec<String>,
    pub(super) predicate_name_to_index: HashMap<String, usize>,
    pub(super) goal_predicate_index: Option<usize>,
}

impl Program {
    pub fn new_with_transformations(
        task: Rc<Task>,
        annotation_generator: &AnnotationGenerator,
        transformation_options: &TransformationOptions,
    ) -> Self {
        let mut program = Self::new(task.clone(), annotation_generator);

        if transformation_options.remove_action_predicates {
            program = remove_action_predicates(program);
        }
        program = convert_rules_to_normal_form(program);
        program = add_goal_rule(program, task, annotation_generator);

        program = generate_static_facts(program);

        // always do this last
        program.assign_rule_indices();

        program
    }

    #[cfg(test)]
    pub fn new_raw_for_tests(task: Rc<Task>, annotation_generator: &AnnotationGenerator) -> Self {
        Self::new(task, annotation_generator)
    }

    /// Generate a program for the given task. This is intentionally not public
    /// because users should use [`Self::new_with_transformations`] instead.
    fn new(task: Rc<Task>, annotation_generator: &AnnotationGenerator) -> Self {
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
                annotation_generator,
                task.clone(),
            ));

            rules.append(&mut Self::generate_action_effect_rules(
                action_schema,
                &mut predicate_name_to_index,
                annotation_generator,
                task.clone(),
            ));
        }

        Self {
            static_facts: vec![],
            rules,
            task,
            predicate_names,
            predicate_name_to_index,
            goal_predicate_index: None,
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
            .map(|p| {
                if p.is_negative() {
                    panic!("Negative preconditions are not supported for Datalog");
                } else {
                    Atom::new_from_atom_schema(p.underlying())
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
                if e.is_negative() {
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

    /// Create a new auxillary predicate, do all the bookkeeping and return the
    /// index of the predicate. If name is provided, will check that it is not
    /// already in use.
    pub(super) fn new_auxillary_predicate(&mut self, name: Option<String>) -> usize {
        let index = self.predicate_names.len();
        let name = match name {
            Some(name) => {
                let name = format!("@{}", name);
                assert!(
                    !self.predicate_name_to_index.contains_key(&name),
                    "Predicate name {} already exists",
                    name
                );
                name
            }
            None => format!("@p{}", index),
        };
        self.predicate_names.push(name.clone());
        self.predicate_name_to_index.insert(name, index);
        index
    }

    fn assign_rule_indices(&mut self) {
        for (i, rule) in self.rules.iter_mut().enumerate() {
            rule.set_index(RuleIndex::new(i));
        }
    }
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        self.rules == other.rules
            && self.predicate_name_to_index == other.predicate_name_to_index
            && self.predicate_names == other.predicate_names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::datalog::Annotation;
    use crate::search::Task;
    use crate::test_utils::*;

    #[test]
    fn test_new_raw_program() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let program = Program::new_raw_for_tests(task.clone(), &annotation_generator);

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
}
