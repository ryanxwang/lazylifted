use crate::search::{heuristics::Heuristic, DBState, GoalAtom, Task};

pub struct GoalCounting;

impl GoalCounting {
    pub fn new() -> Self {
        GoalCounting {}
    }
}

impl Heuristic for GoalCounting {
    fn evaluate(&mut self, state: &DBState, task: &Task) -> f64 {
        let mut unsatisfied_goal_count = 0;
        for goal_atom in &task.goal.atoms {
            if !is_goal_atom_satisfied(goal_atom, state) {
                unsatisfied_goal_count += 1;
            }
        }
        for &pred in &task.goal.positive_nullary_goals {
            if !state.nullary_atoms[pred] {
                unsatisfied_goal_count += 1;
            }
        }
        for &pred in &task.goal.negative_nullary_goals {
            if state.nullary_atoms[pred] {
                unsatisfied_goal_count += 1;
            }
        }
        unsatisfied_goal_count as f64
    }
}

fn is_goal_atom_satisfied(goal_atom: &GoalAtom, state: &DBState) -> bool {
    let achieved = state.relations[goal_atom.predicate_index]
        .tuples
        .contains(&goal_atom.arguments);
    (achieved && !goal_atom.negated) || (!achieved && goal_atom.negated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn goal_counting() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let mut heuristic = GoalCounting {};
        let state = task.initial_state.clone();
        assert_eq!(heuristic.evaluate(&state, &task), 4.0);
    }
}
