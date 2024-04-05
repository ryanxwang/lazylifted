use crate::search::states::GroundAtom;
use crate::search::successor_generators::{
    JoinAlgorithm, PrecompiledActionData, SuccessorGenerator,
};
use crate::search::{Action, ActionSchema, DBState, SchemaAtom, Task};

pub struct JoinSuccessorGenerator<T>
where
    T: JoinAlgorithm,
{
    join_algorithm: T,
    action_data: Vec<PrecompiledActionData>,
}

impl<T> JoinSuccessorGenerator<T>
where
    T: JoinAlgorithm,
{
    pub fn new(join_algorithm: T, task: &Task) -> Self {
        let action_data = task
            .action_schemas
            .iter()
            .map(|action_schema| precompile_action_data(action_schema))
            .collect();

        Self {
            join_algorithm,
            action_data,
        }
    }
}

impl<T> SuccessorGenerator for JoinSuccessorGenerator<T>
where
    T: JoinAlgorithm,
{
    fn get_applicable_actions(&self, state: &DBState, action: &ActionSchema) -> Vec<Action> {
        if is_trivially_inapplicable(action, state) {
            return vec![];
        }
        if action.is_ground() {
            if is_ground_action_applicable(action, state) {
                return vec![Action {
                    index: action.index,
                    instantiation: vec![],
                }];
            } else {
                return vec![];
            }
        }

        let instantiations = self
            .join_algorithm
            .instantiate(state, &self.action_data[action.index]);

        if instantiations.tuples.is_empty() {
            return vec![];
        }

        let mut free_var_indices = vec![];
        let mut map_indices_to_position = vec![];

        for (i, &parameter) in instantiations.tuple_index.iter().enumerate() {
            if instantiations.index_is_variable(i) {
                free_var_indices.push(parameter as usize);
                map_indices_to_position.push(i);
            }
        }

        instantiations
            .tuples
            .iter()
            .map(|tuple| {
                let mut ordered_tuple = vec![0; free_var_indices.len()];
                for i in 0..free_var_indices.len() {
                    ordered_tuple[free_var_indices[i]] = tuple[map_indices_to_position[i]];
                }
                Action {
                    index: action.index,
                    instantiation: ordered_tuple,
                }
            })
            .collect()
    }

    fn generate_successor(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        action: &Action,
    ) -> DBState {
        let mut new_state = state.clone();
        for i in 0..action_schema.positive_nullary_effects.len() {
            if action_schema.positive_nullary_effects[i] {
                new_state.nullary_atoms[i] = true;
            }
            if action_schema.negative_nullary_effects[i] {
                new_state.nullary_atoms[i] = false;
            }
        }

        debug_assert!(action_schema
            .effects
            .iter()
            .all(|effect| effect.predicate_index
                == new_state.relations[effect.predicate_index].predicate_symbol));

        if action_schema.is_ground() {
            for effect in &action_schema.effects {
                let atom = effect
                    .arguments
                    .iter()
                    .map(|arg| {
                        debug_assert!(arg.is_constant());
                        arg.get_index()
                    })
                    .collect();

                if effect.negated {
                    new_state.relations[effect.predicate_index]
                        .tuples
                        .remove(&atom);
                } else {
                    new_state.relations[effect.predicate_index]
                        .tuples
                        .insert(atom);
                }
            }
        } else {
            for effect in &action_schema.effects {
                let atom = instantiate_effect(effect, action);
                if effect.negated {
                    new_state.relations[effect.predicate_index]
                        .tuples
                        .remove(&atom);
                } else {
                    new_state.relations[effect.predicate_index]
                        .tuples
                        .insert(atom);
                }
            }
        }

        new_state
    }
}

fn precompile_action_data(action_schema: &ActionSchema) -> PrecompiledActionData {
    let relevant_precondition_atoms = action_schema
        .preconditions
        .iter()
        .filter_map(|p| {
            if p.is_nullary() {
                None
            } else {
                Some(p.clone())
            }
        })
        .collect();

    PrecompiledActionData {
        is_ground: action_schema.is_ground(),
        relevant_precondition_atoms,
    }
}

fn is_ground_action_applicable(action: &ActionSchema, state: &DBState) -> bool {
    for precondition in &action.preconditions {
        let index = precondition.predicate_index;
        let tuple: Vec<usize> = precondition
            .arguments
            .iter()
            .map(|arg| {
                debug_assert!(arg.is_constant());
                arg.get_index()
            })
            .collect();

        let tuples_in_relation = &state.relations[index].tuples;
        if tuples_in_relation.contains(&tuple) == precondition.negated {
            // Either this is a negative precondition and the tuple is present
            // or this is a positive precondition and the tuple is not present
            return false;
        }
    }

    true
}

fn is_trivially_inapplicable(action: &ActionSchema, state: &DBState) -> bool {
    let positive_precond = &action.positive_nullary_preconditions;
    let negative_precond = &action.negative_nullary_preconditions;
    let nullary_atoms = &state.nullary_atoms;
    for i in 0..positive_precond.len() {
        if positive_precond[i] && !nullary_atoms[i] {
            return true;
        }
        if negative_precond[i] && nullary_atoms[i] {
            return true;
        }
    }
    false
}

fn instantiate_effect(effect: &SchemaAtom, action: &Action) -> GroundAtom {
    effect
        .arguments
        .iter()
        .map(|arg| {
            if arg.is_constant() {
                arg.get_index()
            } else {
                action.instantiation[arg.get_index()]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{search::successor_generators::NaiveJoinAlgorithm, test_utils::*};

    #[test]
    fn test_precompile_action_data() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        // should be the pickup action
        let action_data = precompile_action_data(&task.action_schemas[0]);

        assert_eq!(action_data.is_ground, false);
        assert_eq!(action_data.relevant_precondition_atoms.len(), 2); // number of non-nullary preconditions
    }

    #[test]
    fn applicable_actions_in_blocksworld_init() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let generator = JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), &task);

        let state = &task.initial_state;

        // pickup is not applicable in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[0]);
        assert_eq!(actions.len(), 0);

        // putdown is not applicable in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[1]);
        assert_eq!(actions.len(), 0);

        // stack is not applicable in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[2]);
        assert_eq!(actions.len(), 0);

        // unstack is the only applicable action in the initial state
        let actions = generator.get_applicable_actions(state, &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].index, 3);
        assert_eq!(actions[0].instantiation, vec![0, 1]);
    }

    #[test]
    fn successor_generation_in_blocksworld() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        let generator = JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), &task);

        let mut states = Vec::new();
        states.push(task.initial_state);

        // action: (unstack b1 b2)
        let actions = generator.get_applicable_actions(&states[0], &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[0], &task.action_schemas[3], &actions[0]));

        // state: (clear b2, on-table b4, holding b1, on b2 b3, on b3 b4)
        assert_eq!(
            format!("{}", states[1]),
            "(0 [1])(1 [3])(3 [0])(4 [1, 2])(4 [2, 3])"
        );

        // action: (putdown b1)
        let actions = generator.get_applicable_actions(&states[1], &task.action_schemas[1]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[1], &task.action_schemas[1], &actions[0]));

        // state: (clear b1, clear b2, on-table b1, on-table b4, arm-empty, on b2 b3, on b3 b4)
        assert_eq!(
            format!("{}", states[2]),
            "(0 [0])(0 [1])(1 [0])(1 [3])(4 [1, 2])(4 [2, 3])(2)"
        );

        // action: (unstack b2 b3)
        let actions = generator.get_applicable_actions(&states[2], &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[2], &task.action_schemas[3], &actions[0]));

        // state: (clear b1, clear b3, on-table b1, on-table b4, holding b2, on b3 b4)
        assert_eq!(
            format!("{}", states[3]),
            "(0 [0])(0 [2])(1 [0])(1 [3])(3 [1])(4 [2, 3])"
        );

        // action: (putdown b2)
        let actions = generator.get_applicable_actions(&states[3], &task.action_schemas[1]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[3], &task.action_schemas[1], &actions[0]));

        // state: (clear b1, clear b2, clear b3, on-table b1, on-table b2, on-table b4, arm-empty, on b3 b4)
        assert_eq!(
            format!("{}", states[4]),
            "(0 [0])(0 [1])(0 [2])(1 [0])(1 [1])(1 [3])(4 [2, 3])(2)"
        );

        // action: (unstack b3 b4)
        let actions = generator.get_applicable_actions(&states[4], &task.action_schemas[3]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[4], &task.action_schemas[3], &actions[0]));

        // state: (clear b1, clear b2, clear b4, on-table b1, on-table b2, on-table b4, holding b3)
        assert_eq!(
            format!("{}", states[5]),
            "(0 [0])(0 [1])(0 [3])(1 [0])(1 [1])(1 [3])(3 [2])"
        );

        // action: (stack b3 b1)
        let actions = generator.get_applicable_actions(&states[5], &task.action_schemas[2]);
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
        states.push(generator.generate_successor(&states[5], &task.action_schemas[2], action));

        // state: (clear b2, clear b3, clear b4, on-table b1, on-table b2, on-table b4, arm-empty, on b3 b1)
        assert_eq!(
            format!("{}", states[6]),
            "(0 [1])(0 [2])(0 [3])(1 [0])(1 [1])(1 [3])(4 [2, 0])(2)"
        );

        // action: (pickup b2)
        let actions = generator.get_applicable_actions(&states[6], &task.action_schemas[0]);
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
        states.push(generator.generate_successor(&states[6], &task.action_schemas[0], action));

        // state: (clear b3, clear b4, on-table b1, on-table b4, holding b2, on b3 b1)
        assert_eq!(
            format!("{}", states[7]),
            "(0 [2])(0 [3])(1 [0])(1 [3])(3 [1])(4 [2, 0])"
        );

        // action: (stack b2 b3)
        let actions = generator.get_applicable_actions(&states[7], &task.action_schemas[2]);
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
        states.push(generator.generate_successor(&states[7], &task.action_schemas[2], action));

        // state: (clear b2, clear b4, on-table b1, on-table b4, arm-empty, on b2 b3, on b3 b1)
        assert_eq!(
            format!("{}", states[8]),
            "(0 [1])(0 [3])(1 [0])(1 [3])(4 [1, 2])(4 [2, 0])(2)"
        );

        // action: (pickup b4)
        let actions = generator.get_applicable_actions(&states[8], &task.action_schemas[0]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[8], &task.action_schemas[0], &actions[0]));

        // state: (clear b2, on-table b1, holding b4, on b2 b3, on b3 b1)
        assert_eq!(
            format!("{}", states[9]),
            "(0 [1])(1 [0])(3 [3])(4 [1, 2])(4 [2, 0])"
        );

        // action: (stack b4 b2)
        let actions = generator.get_applicable_actions(&states[9], &task.action_schemas[2]);
        assert_eq!(actions.len(), 1);
        states.push(generator.generate_successor(&states[9], &task.action_schemas[2], &actions[0]));

        // state: (clear b4, on-table b1, arm-empty, on b2 b3, on b3 b1, on b4 b2)
        assert_eq!(
            format!("{}", states[10]),
            "(0 [3])(1 [0])(4 [1, 2])(4 [2, 0])(4 [3, 1])(2)"
        );
    }
}
