use crate::search::successor_generators::{JoinAlgorithm, PrecompiledActionData};
use crate::search::{Action, ActionSchema, DBState, Task};

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

    pub fn get_applicable_actions(&self, state: &DBState, action: &ActionSchema) -> Vec<Action> {
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
                assert!(arg.is_constant());
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
}
