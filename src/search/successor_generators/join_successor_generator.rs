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
            .all(|effect| effect.predicate_index()
                == new_state.relations[effect.predicate_index()].predicate_symbol));

        if action_schema.is_ground() {
            for effect in &action_schema.effects {
                let atom = effect
                    .arguments()
                    .iter()
                    .map(|arg| {
                        debug_assert!(arg.is_constant());
                        arg.get_index()
                    })
                    .collect();

                if effect.is_negated() {
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
            for effect in &action_schema.effects {
                let atom = instantiate_effect(effect, action);
                if effect.is_negated() {
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

fn precompile_action_data(action_schema: &ActionSchema) -> PrecompiledActionData {
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

    PrecompiledActionData {
        action_index: action_schema.index,
        is_ground: action_schema.is_ground(),
        relevant_precondition_atoms,
    }
}

fn is_ground_action_applicable(action: &ActionSchema, state: &DBState) -> bool {
    for precondition in action.preconditions() {
        let index = precondition.predicate_index();
        let tuple: Vec<usize> = precondition
            .arguments()
            .iter()
            .map(|arg| {
                debug_assert!(arg.is_constant());
                arg.get_index()
            })
            .collect();

        let tuples_in_relation = &state.relations[index].tuples;
        if tuples_in_relation.contains(&tuple) == precondition.is_negated() {
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
        .arguments()
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
    use crate::test_utils::*;

    #[test]
    fn test_precompile_action_data() {
        let task = Task::from_text(BLOCKSWORLD_DOMAIN_TEXT, BLOCKSWORLD_PROBLEM13_TEXT);
        // should be the pickup action
        let action_data = precompile_action_data(&task.action_schemas[0]);

        assert_eq!(action_data.is_ground, false);
        assert_eq!(action_data.relevant_precondition_atoms.len(), 2); // number of non-nullary preconditions
    }
}
