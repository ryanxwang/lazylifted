//! A plan is a sequence of actions that can be executed to achieve a goal. This
//! module provides the [`Plan`] struct, which represents a plan.

use crate::parsed_types::{ActionName, Name};
use crate::parsers::Parser;
use crate::search::{Action, Task};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    steps: Vec<Action>,
}

impl Plan {
    pub fn empty() -> Self {
        Self { steps: vec![] }
    }

    pub fn new(steps: Vec<Action>) -> Self {
        Self { steps }
    }

    pub fn from_path(path: &PathBuf, task: &Task) -> Self {
        let contents = std::fs::read_to_string(path).expect("Failed to read plan file");
        Self::from_text(&contents, task)
    }

    pub fn from_text(text: &str, task: &Task) -> Self {
        let parsed_plan: crate::parsed_types::Plan =
            crate::parsed_types::Plan::from_str(text).expect("Failed to parse plan");

        let action_table: HashMap<ActionName, usize> = task
            .action_schemas()
            .iter()
            .enumerate()
            .map(|(index, action_schema)| (action_schema.name().clone(), index))
            .collect();

        let object_table: HashMap<Name, usize> = task
            .objects
            .iter()
            .enumerate()
            .map(|(index, object)| (object.name.clone(), index))
            .collect();

        let mut steps = vec![];
        for step in parsed_plan.steps() {
            if !action_table.contains_key(step.action_name()) {
                panic!("Action {} not found in task", step.action_name());
            }

            for parameter in step.parameters() {
                if !object_table.contains_key(parameter) {
                    panic!("Object {} not found in task", parameter);
                }
            }

            steps.push(Action {
                index: action_table[&step.action_name()],
                instantiation: step
                    .parameters()
                    .iter()
                    .map(|parameter| object_table[parameter])
                    .collect(),
            });
        }

        Self { steps }
    }

    pub fn steps(&self) -> &[Action] {
        &self.steps
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn to_string(&self, task: &Task) -> String {
        self.steps
            .iter()
            .map(|action| action.to_string(task))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl IntoIterator for Plan {
    type Item = Action;
    type IntoIter = std::vec::IntoIter<Action>;

    fn into_iter(self) -> Self::IntoIter {
        self.steps.into_iter()
    }
}

impl Deref for Plan {
    type Target = [Action];

    fn deref(&self) -> &Self::Target {
        &self.steps
    }
}

impl DerefMut for Plan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn from_text_works() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let plan_text = r#"(unstack b1 b2)
        (putdown b1)
        (unstack b2 b3)
        (putdown b2)
        (unstack b3 b4)
        (stack b3 b1)
        (pickup b2)
        (stack b2 b3)
        (pickup b4)
        (stack b4 b2)
        ; cost = 10 (unit cost)
        "#;

        let plan = Plan::from_text(plan_text, &task);
        assert_eq!(plan.len(), 10);

        assert_eq!(plan.steps[0].index, 3);
        assert_eq!(plan.steps[0].instantiation, vec![0, 1]);

        assert_eq!(plan.steps[1].index, 1);
        assert_eq!(plan.steps[1].instantiation, vec![0]);

        assert_eq!(plan.steps[2].index, 3);
        assert_eq!(plan.steps[2].instantiation, vec![1, 2]);

        assert_eq!(plan.steps[3].index, 1);
        assert_eq!(plan.steps[3].instantiation, vec![1]);

        assert_eq!(plan.steps[4].index, 3);
        assert_eq!(plan.steps[4].instantiation, vec![2, 3]);

        assert_eq!(plan.steps[5].index, 2);
        assert_eq!(plan.steps[5].instantiation, vec![2, 0]);

        assert_eq!(plan.steps[6].index, 0);
        assert_eq!(plan.steps[6].instantiation, vec![1]);

        assert_eq!(plan.steps[7].index, 2);
        assert_eq!(plan.steps[7].instantiation, vec![1, 2]);

        assert_eq!(plan.steps[8].index, 0);
        assert_eq!(plan.steps[8].instantiation, vec![3]);

        assert_eq!(plan.steps[9].index, 2);
        assert_eq!(plan.steps[9].instantiation, vec![3, 1]);
    }
}
