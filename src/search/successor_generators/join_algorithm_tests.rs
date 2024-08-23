use crate::search::PartialAction;
use crate::search::{successor_generators::SuccessorGeneratorName, Action, Task};
use crate::test_utils::*;

pub fn test_applicable_actions_in_blocksworld_init(name: SuccessorGeneratorName) {
    let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
    let generator = name.create(&task);

    let state = &task.initial_state;

    // pickup is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[0]);
    assert_eq!(actions.len(), 0);

    // putdown is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[1]);
    assert_eq!(actions.len(), 0);

    // stack is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[2]);
    assert_eq!(actions.len(), 0);

    // unstack is the only applicable action in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[3]);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].index, 3);
    assert_eq!(actions[0].instantiation, vec![0, 1]);
}

pub fn test_applicable_actions_from_partial_in_blocksworld(name: SuccessorGeneratorName) {
    let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
    let generator = name.create(&task);

    let mut state = task.initial_state.clone();

    // (unstack b1 b2)
    let actions = generator.get_applicable_actions(&state, &task.action_schemas()[3]);
    state = generator.generate_successor(&state, &task.action_schemas()[3], &actions[0]);

    // (putdown b1)
    let actions = generator.get_applicable_actions(&state, &task.action_schemas()[1]);
    state = generator.generate_successor(&state, &task.action_schemas()[1], &actions[0]);

    // (unstack b2 b3)
    let actions = generator.get_applicable_actions(&state, &task.action_schemas()[3]);
    state = generator.generate_successor(&state, &task.action_schemas()[3], &actions[0]);

    // we can now either stack b2 on b1 or b3

    // fixing none of the parameters should yield both
    let actions = generator.get_applicable_actions_from_partial(
        &state,
        &task.action_schemas()[2],
        &PartialAction::new(2, vec![]),
    );
    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions,
        vec![Action::new(2, vec![1, 0]), Action::new(2, vec![1, 2])]
    );

    // further restricting the first argumenting shouldn't change anything
    let actions = generator.get_applicable_actions_from_partial(
        &state,
        &task.action_schemas()[2],
        &PartialAction::new(2, vec![1]),
    );
    assert_eq!(actions.len(), 2);
    assert_eq!(
        actions,
        vec![Action::new(2, vec![1, 0]), Action::new(2, vec![1, 2])]
    );

    // fixing the last one should
    let actions = generator.get_applicable_actions_from_partial(
        &state,
        &task.action_schemas()[2],
        &PartialAction::new(2, vec![1, 0]),
    );
    assert_eq!(actions.len(), 1);
    assert_eq!(actions, vec![Action::new(2, vec![1, 0])]);
}

pub fn test_successor_generation_in_blocksworld(name: SuccessorGeneratorName) {
    let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
    let generator = name.create(&task);

    let mut states = Vec::new();
    states.push(task.initial_state.clone());

    // action: (unstack b1 b2)
    let actions = generator.get_applicable_actions(&states[0], &task.action_schemas()[3]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[0], &task.action_schemas()[3], &actions[0]));

    // state: (clear b2, on-table b4, holding b1, on b2 b3, on b3 b4)
    assert_eq!(
        format!("{}", states[1]),
        "(0 [1])(1 [3])(3 [0])(4 [1, 2])(4 [2, 3])"
    );

    // action: (putdown b1)
    let actions = generator.get_applicable_actions(&states[1], &task.action_schemas()[1]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[1], &task.action_schemas()[1], &actions[0]));

    // state: (clear b1, clear b2, on-table b1, on-table b4, arm-empty, on b2 b3, on b3 b4)
    assert_eq!(
        format!("{}", states[2]),
        "(0 [0])(0 [1])(1 [0])(1 [3])(4 [1, 2])(4 [2, 3])(2)"
    );

    // action: (unstack b2 b3)
    let actions = generator.get_applicable_actions(&states[2], &task.action_schemas()[3]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[2], &task.action_schemas()[3], &actions[0]));

    // state: (clear b1, clear b3, on-table b1, on-table b4, holding b2, on b3 b4)
    assert_eq!(
        format!("{}", states[3]),
        "(0 [0])(0 [2])(1 [0])(1 [3])(3 [1])(4 [2, 3])"
    );

    // action: (putdown b2)
    let actions = generator.get_applicable_actions(&states[3], &task.action_schemas()[1]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[3], &task.action_schemas()[1], &actions[0]));

    // state: (clear b1, clear b2, clear b3, on-table b1, on-table b2, on-table b4, arm-empty, on b3 b4)
    assert_eq!(
        format!("{}", states[4]),
        "(0 [0])(0 [1])(0 [2])(1 [0])(1 [1])(1 [3])(4 [2, 3])(2)"
    );

    // action: (unstack b3 b4)
    let actions = generator.get_applicable_actions(&states[4], &task.action_schemas()[3]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[4], &task.action_schemas()[3], &actions[0]));

    // state: (clear b1, clear b2, clear b4, on-table b1, on-table b2, on-table b4, holding b3)
    assert_eq!(
        format!("{}", states[5]),
        "(0 [0])(0 [1])(0 [3])(1 [0])(1 [1])(1 [3])(3 [2])"
    );

    // action: (stack b3 b1)
    let actions = generator.get_applicable_actions(&states[5], &task.action_schemas()[2]);
    assert_eq!(actions.len(), 3);
    assert!(actions.contains(&Action {
        // (stack b3 b1)
        index: 2,
        instantiation: vec![2, 0]
    }));
    assert!(actions.contains(&Action {
        // (stack b3 b2)
        index: 2,
        instantiation: vec![2, 1]
    }));
    assert!(actions.contains(&Action {
        // (stack b3 b4)
        index: 2,
        instantiation: vec![2, 3]
    }));
    let action = actions.iter().find(|a| a.instantiation[1] == 0).unwrap();
    states.push(generator.generate_successor(&states[5], &task.action_schemas()[2], action));

    // state: (clear b2, clear b3, clear b4, on-table b1, on-table b2, on-table b4, arm-empty, on b3 b1)
    assert_eq!(
        format!("{}", states[6]),
        "(0 [1])(0 [2])(0 [3])(1 [0])(1 [1])(1 [3])(4 [2, 0])(2)"
    );

    // action: (pickup b2)
    let actions = generator.get_applicable_actions(&states[6], &task.action_schemas()[0]);
    assert_eq!(actions.len(), 2);
    assert!(actions.contains(&Action {
        // (pickup b2)
        index: 0,
        instantiation: vec![1]
    }));
    assert!(actions.contains(&Action {
        // (pickup b4)
        index: 0,
        instantiation: vec![3]
    }));
    let action = actions.iter().find(|a| a.instantiation[0] == 1).unwrap();
    states.push(generator.generate_successor(&states[6], &task.action_schemas()[0], action));

    // state: (clear b3, clear b4, on-table b1, on-table b4, holding b2, on b3 b1)
    assert_eq!(
        format!("{}", states[7]),
        "(0 [2])(0 [3])(1 [0])(1 [3])(3 [1])(4 [2, 0])"
    );

    // action: (stack b2 b3)
    let actions = generator.get_applicable_actions(&states[7], &task.action_schemas()[2]);
    assert_eq!(actions.len(), 2);
    assert!(actions.contains(&Action {
        // (stack b2 b3)
        index: 2,
        instantiation: vec![1, 2]
    }));
    assert!(actions.contains(&Action {
        // (stack b2 b4)
        index: 2,
        instantiation: vec![1, 3]
    }));
    let action = actions.iter().find(|a| a.instantiation[1] == 2).unwrap();
    states.push(generator.generate_successor(&states[7], &task.action_schemas()[2], action));

    // state: (clear b2, clear b4, on-table b1, on-table b4, arm-empty, on b2 b3, on b3 b1)
    assert_eq!(
        format!("{}", states[8]),
        "(0 [1])(0 [3])(1 [0])(1 [3])(4 [1, 2])(4 [2, 0])(2)"
    );

    // action: (pickup b4)
    let actions = generator.get_applicable_actions(&states[8], &task.action_schemas()[0]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[8], &task.action_schemas()[0], &actions[0]));

    // state: (clear b2, on-table b1, holding b4, on b2 b3, on b3 b1)
    assert_eq!(
        format!("{}", states[9]),
        "(0 [1])(1 [0])(3 [3])(4 [1, 2])(4 [2, 0])"
    );

    // action: (stack b4 b2)
    let actions = generator.get_applicable_actions(&states[9], &task.action_schemas()[2]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[9], &task.action_schemas()[2], &actions[0]));

    // state: (clear b4, on-table b1, arm-empty, on b2 b3, on b3 b1, on b4 b2)
    assert_eq!(
        format!("{}", states[10]),
        "(0 [3])(1 [0])(4 [1, 2])(4 [2, 0])(4 [3, 1])(2)"
    );
}

pub fn test_applicable_actions_in_spanner_init(name: SuccessorGeneratorName) {
    let task = Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT);
    let generator = name.create(&task);

    let state = &task.initial_state;

    // (walk shed location1 bob)
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[0]);
    assert_eq!(actions.len(), 1);

    // pickup_spanner is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[1]);
    assert_eq!(actions.len(), 0);

    // tighten_nut is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[2]);
    assert_eq!(actions.len(), 0);
}

pub fn test_applicable_actions_in_ferry_init(name: SuccessorGeneratorName) {
    let task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
    let generator = name.create(&task);

    let state = &task.initial_state;

    // (sail loc1 loc2), (sail loc1 loc3)
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[0]);
    assert_eq!(actions.len(), 2);

    // board is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[1]);
    assert_eq!(actions.len(), 0);

    // debark is not applicable in the initial state
    let actions = generator.get_applicable_actions(state, &task.action_schemas()[2]);
    assert_eq!(actions.len(), 0);
}

pub fn test_successor_generation_in_ferry(name: SuccessorGeneratorName) {
    let task = Task::from_text(FERRY_DOMAIN_TEXT, FERRY_PROBLEM10_TEXT);
    let generator = name.create(&task);

    let mut states = Vec::new();
    states.push(task.initial_state.clone());

    // action: (sail loc1 loc2)
    let actions = generator.get_applicable_actions(&states[0], &task.action_schemas()[0]);
    assert_eq!(actions.len(), 2);
    let action = actions.iter().find(|a| a.instantiation[1] == 3).unwrap();
    states.push(generator.generate_successor(&states[0], &task.action_schemas()[0], action));

    // state: (at-ferry loc2, at car1 loc2, at car2 loc3, empty-ferry)
    assert_eq!(format!("{}", states[1]), "(0 [3])(1 [0, 3])(1 [1, 4])(2)");

    // action: (board car1 loc2)
    let actions = generator.get_applicable_actions(&states[1], &task.action_schemas()[1]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[1], &task.action_schemas()[1], &actions[0]));

    // state: (at-ferry loc2, at car2 loc3, on car1)
    assert_eq!(format!("{}", states[2]), "(0 [3])(1 [1, 4])(3 [0])");

    // action: (sail loc2 loc1)
    let actions = generator.get_applicable_actions(&states[2], &task.action_schemas()[0]);
    assert_eq!(actions.len(), 2);
    let action = actions.iter().find(|a| a.instantiation[1] == 2).unwrap();
    states.push(generator.generate_successor(&states[2], &task.action_schemas()[0], action));

    // state: (at-ferry loc1, at car2 loc3, on car1)
    assert_eq!(format!("{}", states[3]), "(0 [2])(1 [1, 4])(3 [0])");

    // action: (debark car1 loc1)
    let actions = generator.get_applicable_actions(&states[3], &task.action_schemas()[2]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[3], &task.action_schemas()[2], &actions[0]));

    // state: (at-ferry loc1, at car1 loc1, at car2 loc3, empty-ferry)
    assert_eq!(format!("{}", states[4]), "(0 [2])(1 [0, 2])(1 [1, 4])(2)");

    // action: (sail loc1 loc3)
    let actions = generator.get_applicable_actions(&states[4], &task.action_schemas()[0]);
    assert_eq!(actions.len(), 2);
    let action = actions.iter().find(|a| a.instantiation[1] == 4).unwrap();
    states.push(generator.generate_successor(&states[4], &task.action_schemas()[0], action));

    // state: (at-ferry loc3, at car1 loc1, at car2 loc3, empty-ferry)
    assert_eq!(format!("{}", states[5]), "(0 [4])(1 [0, 2])(1 [1, 4])(2)");

    // action: (board car2 loc3)
    let actions = generator.get_applicable_actions(&states[5], &task.action_schemas()[1]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[5], &task.action_schemas()[1], &actions[0]));

    // state: (at-ferry loc3, at car1 loc1, on car2)
    assert_eq!(format!("{}", states[6]), "(0 [4])(1 [0, 2])(3 [1])");

    // action: (sail loc3 loc1)
    let actions = generator.get_applicable_actions(&states[6], &task.action_schemas()[0]);
    assert_eq!(actions.len(), 2);
    let action = actions.iter().find(|a| a.instantiation[1] == 2).unwrap();
    states.push(generator.generate_successor(&states[6], &task.action_schemas()[0], action));

    // state: (at-ferry loc1, at car1 loc1, on car2)
    assert_eq!(format!("{}", states[7]), "(0 [2])(1 [0, 2])(3 [1])");

    // action: (debark car2 loc1)
    let actions = generator.get_applicable_actions(&states[7], &task.action_schemas()[2]);
    assert_eq!(actions.len(), 1);
    states.push(generator.generate_successor(&states[7], &task.action_schemas()[2], &actions[0]));

    // state: (at-ferry loc1, at car1 loc1, at car2 loc1, empty-ferry)
    assert_eq!(format!("{}", states[8]), "(0 [2])(1 [0, 2])(1 [1, 2])(2)");
}
