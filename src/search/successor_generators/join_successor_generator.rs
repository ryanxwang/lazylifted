use crate::search::successor_generators::{
    JoinAlgorithm, PrecompiledActionData, SuccessorGenerator,
};
use crate::search::{
    Action, ActionSchema, AtomSchema, DBState, Negatable, PartialAction, RawSmallTuple, SmallTuple,
    Task, NO_PARTIAL,
};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug)]
pub struct JoinSuccessorGenerator<T>
where
    T: JoinAlgorithm + Debug,
{
    join_algorithm: T,
    action_data: Vec<PrecompiledActionData>,
}

impl<T> JoinSuccessorGenerator<T>
where
    T: JoinAlgorithm + Debug,
{
    pub fn new(join_algorithm: T, task: &Task) -> Self {
        let action_data = task
            .action_schemas()
            .iter()
            .map(|action_schema| precompile_action_data(task, action_schema))
            .collect();

        Self {
            join_algorithm,
            action_data,
        }
    }
}

impl<T> SuccessorGenerator for JoinSuccessorGenerator<T>
where
    T: JoinAlgorithm + Debug,
{
    fn get_applicable_actions(&self, state: &DBState, action_schema: &ActionSchema) -> Vec<Action> {
        self.get_applicable_actions_from_partial(state, action_schema, &NO_PARTIAL)
    }

    fn get_applicable_actions_from_partial(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        partial_action: &PartialAction,
    ) -> Vec<Action> {
        if is_trivially_inapplicable(action_schema, state) {
            return vec![];
        }
        if action_schema.is_ground() {
            if is_ground_action_applicable(action_schema, state) {
                return vec![Action {
                    index: action_schema.index(),
                    instantiation: vec![],
                }];
            } else {
                return vec![];
            }
        }

        let fixed_schema_params = if *partial_action == NO_PARTIAL {
            HashMap::new()
        } else {
            assert_eq!(partial_action.schema_index(), action_schema.index());
            partial_action
                .partial_instantiation()
                .iter()
                .enumerate()
                .map(|(param_index, &object_index)| (param_index, object_index))
                .collect()
        };

        let instantiations = self.join_algorithm.instantiate(
            state,
            &self.action_data[action_schema.index()],
            &fixed_schema_params,
        );

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

        let actions = instantiations
            .tuples
            .iter()
            .map(|tuple| {
                let mut ordered_tuple = vec![0; free_var_indices.len()];
                for i in 0..free_var_indices.len() {
                    ordered_tuple[free_var_indices[i]] = tuple[map_indices_to_position[i]];
                }
                Action {
                    index: action_schema.index(),
                    instantiation: ordered_tuple,
                }
            })
            .collect();

        actions
    }

    fn generate_successor(
        &self,
        state: &DBState,
        action_schema: &ActionSchema,
        action: &Action,
    ) -> DBState {
        let mut new_state = state.clone();

        for effect in action_schema.effects() {
            if !effect.is_nullary() {
                // dealt later
                continue;
            }

            let index = effect.predicate_index();
            new_state.nullary_atoms[index] = effect.is_positive();
        }

        assert!(action_schema
            .effects()
            .iter()
            .all(|effect| effect.predicate_index()
                == new_state.relations[effect.predicate_index()].predicate_symbol));

        if action_schema.is_ground() {
            for effect in action_schema.effects() {
                if effect.is_nullary() {
                    continue;
                }
                let atom = effect
                    .arguments()
                    .iter()
                    .map(|arg| {
                        assert!(arg.is_constant());
                        arg.get_index()
                    })
                    .collect::<RawSmallTuple>()
                    .into();

                if effect.is_negative() {
                    new_state.relations[effect.predicate_index()]
                        .tuples
                        .remove(&atom);
                } else {
                    new_state.relations[effect.predicate_index()]
                        .tuples
                        .insert(atom);
                }
            }
        } else {
            for effect in action_schema.effects() {
                if effect.is_nullary() {
                    continue;
                }
                let atom = instantiate_effect(effect, action);
                if effect.is_negative() {
                    new_state.relations[effect.predicate_index()]
                        .tuples
                        .remove(&atom);
                } else {
                    new_state.relations[effect.predicate_index()]
                        .tuples
                        .insert(atom);
                }
            }
        }

        new_state
    }
}

fn precompile_action_data(task: &Task, action_schema: &ActionSchema) -> PrecompiledActionData {
    let relevant_precondition_atoms = action_schema
        .preconditions()
        .iter()
        .filter_map(|p| {
            if p.is_nullary() {
                None
            } else {
                Some(p.clone())
            }
        })
        .collect();

    let objects_per_param = action_schema
        .parameters()
        .iter()
        .map(|param| {
            task.objects_per_type()
                .get(param.type_index())
                .unwrap()
                .clone()
        })
        .collect();

    PrecompiledActionData {
        action_index: action_schema.index(),
        is_ground: action_schema.is_ground(),
        relevant_precondition_atoms,
        objects_per_param,
    }
}

fn is_ground_action_applicable(action: &ActionSchema, state: &DBState) -> bool {
    for precondition in action.preconditions() {
        let index = precondition.predicate_index();
        let tuple: SmallTuple = precondition
            .arguments()
            .iter()
            .map(|arg| {
                assert!(arg.is_constant());
                arg.get_index()
            })
            .collect::<RawSmallTuple>()
            .into();

        let tuples_in_relation = &state.relations[index].tuples;
        if tuples_in_relation.contains(&tuple) == precondition.is_negative() {
            // Either this is a negative precondition and the tuple is present
            // or this is a positive precondition and the tuple is not present
            return false;
        }
    }

    true
}

fn is_trivially_inapplicable(action: &ActionSchema, state: &DBState) -> bool {
    for precond in action.preconditions() {
        if !precond.is_nullary() {
            continue;
        }

        let index = precond.predicate_index();
        if precond.is_negative() && state.nullary_atoms[index] {
            return true;
        }
        if precond.is_positive() && !state.nullary_atoms[index] {
            return true;
        }
    }

    false
}

fn instantiate_effect(effect: &Negatable<AtomSchema>, action: &Action) -> SmallTuple {
    effect
        .arguments()
        .iter()
        .map(|arg| {
            if arg.is_constant() {
                arg.get_index()
            } else {
                action.instantiation[arg.get_index()]
            }
        })
        .collect::<RawSmallTuple>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn test_precompile_action_data() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        // should be the pickup action
        let action_data = precompile_action_data(&task, &task.action_schemas()[0]);

        assert!(!action_data.is_ground);
        assert_eq!(action_data.relevant_precondition_atoms.len(), 2); // number of non-nullary preconditions
        assert_eq!(action_data.objects_per_param.len(), 1); // number of parameters
    }
}
