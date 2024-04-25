use crate::search::{
    heuristics::{Heuristic, HeuristicValue},
    DBState, Task,
};

#[derive(Debug)]
pub struct GoalCounting {}

impl GoalCounting {
    pub fn new() -> Self {
        GoalCounting {}
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn goal_counting() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let mut heuristic = GoalCounting::new();
        let state = task.initial_state.clone();
        assert_eq!(heuristic.evaluate(&state, &task), 4.0);
    }
}
