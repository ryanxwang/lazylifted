use crate::search::{Plan, SuccessorGenerator, Task};

pub fn validate(
    plan: &Plan,
    generator: &dyn SuccessorGenerator,
    task: &Task,
) -> Result<(), String> {
    let mut cur_state = task.initial_state.clone();
    for action in plan.steps() {
        let action_schema = &task.action_schemas()[action.index];
        let applicable_actions = generator.get_applicable_actions(&cur_state, action_schema);

        if !applicable_actions.contains(action) {
            return Err(format!(
                "Action {} is not applicable in state {:?}",
                action.to_string(task),
                cur_state
            ));
        }

        cur_state = generator.generate_successor(&cur_state, action_schema, action);
    }

    if !task.goal.is_satisfied(&cur_state) {
        return Err(format!(
            "Plan does not reach goal state, final state is: {:?}",
            cur_state
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{search::successor_generators::SuccessorGeneratorName, test_utils::*};

    fn validate_plan(plan: &str) -> Result<(), String> {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let generator = SuccessorGeneratorName::create(&SuccessorGeneratorName::FullReducer, &task);
        let plan = Plan::from_text(plan, &task);
        validate(&plan, generator.as_ref(), &task)
    }

    #[test]
    fn validate_good_plan_ok() {
        let plan = r#"
        (unstack b1 b2)
        (putdown b1)
        (unstack b2 b3)
        (putdown b2)
        (unstack b3 b4)
        (stack b3 b1)
        (pickup b2)
        (stack b2 b3)
        (pickup b4)
        (stack b4 b2)
        "#;

        assert!(validate_plan(plan).is_ok());
    }

    #[test]
    fn validate_bad_plan_not_applicable() {
        let plan = r#"
        (unstack b1 b2)
        (putdown b1)
        (unstack b2 b3)
        (putdown b2)
        (unstack b3 b4)
        (stack b3 b1)
        (pickup b2)
        (stack b2 b3)
        (pickup b4)
        (stack b4 b2)
        (stack b4 b2)
        "#;

        assert!(validate_plan(plan).is_err());
    }

    #[test]
    fn validate_bad_plan_incomplete() {
        let plan = r#"
        (unstack b1 b2)
        (putdown b1)
        (unstack b2 b3)
        (putdown b2)
        (unstack b3 b4)
        (stack b3 b1)
        (pickup b2)
        (stack b2 b3)
        (pickup b4)
        "#;

        assert!(validate_plan(plan).is_err());
    }
}
