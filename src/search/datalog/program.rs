use std::rc::Rc;

use itertools::Itertools;

use crate::search::{
    datalog::{
        atom::Atom,
        fact::Fact,
        rules::{GenericRule, Rule},
        transformation_options::TransformationOptions,
        AnnotationGenerator, RuleCategory,
    },
    Task,
};

/// If true, the program will panic if a negative precondition is encountered.
/// Otherwise, the precondition will be ignored, which is equivalent to an
/// additional relaxation of the planning problem on top of delete relaxation.
const PANIC_ON_NEGATIVE_PRECONDITIONS: bool = false;

#[derive(Debug)]
pub struct Program {
    rules: Vec<Rule>,
    task: Rc<Task>,
    // Predicate names for the atoms, including ones generated when building the
    // program.
    predicate_names: Vec<String>,
}

impl Program {
    pub fn new_with_transformations(
        task: Rc<Task>,
        annotation_generator: AnnotationGenerator,
        _transformation_options: &TransformationOptions,
    ) -> Self {
        let mut predicate_names: Vec<String> = task
            .predicates
            .iter()
            .map(|p| p.name.clone().to_string())
            .collect();

        let mut rules = vec![];
        for action_schema in task.action_schemas() {
            // the action applicability rule, where we create a new predicate
            // applicable-a for each action schema and add a rule
            // applicable-a <-- pre(a) with weight being the action cost.
            let applicability_predicate_name = format!("applicable-{}", action_schema.name());
            predicate_names.push(applicability_predicate_name);
            let effect = Atom::new_from_action_schema(action_schema, predicate_names.len() - 1);

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
            rules.push(Rule::new_generic(GenericRule::new(
                effect,
                conditions,
                1.0,
                annotation,
                action_schema.index(),
            )));

            // the action effect rules, where we create rules of the form
            // p <-- applicable-a for each p in add(a)

            // TODO-soon: implement this
        }

        Self {
            rules,
            task,
            predicate_names,
        }
    }
}
