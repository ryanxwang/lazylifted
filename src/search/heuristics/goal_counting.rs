use crate::search::{
    heuristics::{Heuristic, HeuristicValue},
    successor_generators::SuccessorGeneratorName,
    Action, Atom, DBState, Negatable, PartialAction, SuccessorGenerator, Task,
};
use std::{collections::HashSet, rc::Rc};

#[derive(Debug)]
pub struct GoalCounting {
    successor_generator: Box<dyn SuccessorGenerator>,
}

impl GoalCounting {
    pub fn new(task: Rc<Task>, successor_generator_name: SuccessorGeneratorName) -> Self {
        let successor_generator = successor_generator_name.create(&task);
        GoalCounting {
            successor_generator,
        }
    }
}

impl Heuristic<DBState> for GoalCounting {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> HeuristicValue {
        let mut unsatisfied_goal_count = 0;
        for goal_atom in task.goal.atoms() {
            if !state.satisfied(goal_atom) {
                unsatisfied_goal_count += 1;
            }
        }
        unsatisfied_goal_count.into()
    }
}

impl Heuristic<(DBState, PartialAction)> for GoalCounting {
    fn evaluate(
        &mut self,
        (state, partial): &(DBState, PartialAction),
        task: &Task,
    ) -> HeuristicValue {
        let action_schema = &task.action_schemas()[partial.schema_index()];
        let applicable_actions: Vec<Action> = self
            .successor_generator
            .get_applicable_actions(state, action_schema)
            .into_iter()
            .filter(|action| partial.is_superset_of_action(action))
            .collect();

        let partial_effects = partial.get_partial_effects(action_schema, &applicable_actions);
        let unavoidable_effects: HashSet<Negatable<Atom>> = partial_effects.unavoidable_effects;

        let mut unsatisfied_goal_count = 0;
        for goal_atom in task.goal.atoms() {
            if unavoidable_effects.contains(goal_atom) {
                continue;
            }
            if !state.satisfied(goal_atom) {
                unsatisfied_goal_count += 1;
            }
        }

        unsatisfied_goal_count.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn goal_counting_on_states() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let mut heuristic = GoalCounting::new(task.clone(), SuccessorGeneratorName::FullReducer);
        let state = task.initial_state.clone();
        assert_eq!(heuristic.evaluate(&state, &task), 4.0);
    }

    #[test]
    fn goal_counting_on_partials() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let mut heuristic = GoalCounting::new(task.clone(), SuccessorGeneratorName::FullReducer);

        let state = task.initial_state.clone();
        let partial = PartialAction::new(3, vec![3, 1]);
        assert_eq!(heuristic.evaluate(&(state, partial), &task), 4.0);
    }
}
