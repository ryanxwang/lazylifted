use std::collections::HashMap;

use crate::search::datalog::{
    arguments::Arguments,
    atom::Atom,
    program::Program,
    rules::{GenericRule, Rule},
    term::Term,
    Annotation,
};
use itertools::Itertools;

struct Graph {
    nodes: Vec<usize>,
    edges: Vec<Vec<usize>>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
        }
    }

    fn add_node(&mut self, node: usize) {
        self.nodes.push(node);
        self.edges.push(vec![]);
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges[from].push(to);
        self.edges[to].push(from);
    }

    /// Returns the connected components of the graph, each represented as a
    /// vector of node indices.
    fn get_connected_components(&self) -> Vec<Vec<usize>> {
        let mut components = vec![];
        let mut visited = vec![false; self.nodes.len()];

        for node in 0..self.nodes.len() {
            if !visited[node] {
                let mut component = vec![];
                self.dfs(node, &mut visited, &mut component);
                components.push(component);
            }
        }

        components
    }

    fn dfs(&self, node: usize, visited: &mut Vec<bool>, component: &mut Vec<usize>) {
        visited[node] = true;
        component.push(node);

        for &neighbour in &self.edges[node] {
            if !visited[neighbour] {
                self.dfs(neighbour, visited, component);
            }
        }
    }
}

/// Split the conditions of a rule into connected components, and add the new
/// rules created in the process. This means that we analyse the condition atoms
/// as a graph, with edges indicating shared variables.
///
/// - If there is only one connected component, we do nothing.
/// - Otherwise, we create a new rule for each connected component, which allow
///   creating a new auxilary atom for the conditions in that component. The
///   conditions of the original rule are replaced by the auxilary atoms.
pub(super) fn split_into_connected_components(
    mut program: Program,
    target_rule_index: usize,
) -> Program {
    let components = get_components(&program.rules[target_rule_index]);
    if components.len() == 1 {
        return program;
    }

    // We first update the condition indices in the variable source of the rule
    let old_condition_index_to_new = components
        .iter()
        .enumerate()
        .flat_map(|(i, component)| component.iter().map(move |&j| (j, i)))
        .collect::<HashMap<_, _>>();
    program.rules[target_rule_index]
        .variable_source_mut()
        .update_condition_indices(&old_condition_index_to_new);

    let mut new_conditions = vec![];
    let mut new_rules = vec![];
    for (component_index, component) in components.into_iter().enumerate() {
        if component.len() == 1 {
            // no need to update the source anymore, the existing direct entry
            // is good
            new_conditions
                .push(program.rules[target_rule_index].conditions()[component[0]].clone());
        } else {
            // Otherwise, we will need to create a new rule for this component
            let aux_predicate = program.new_auxillary_predicate();
            let new_rule_conditions: Vec<Atom> = component
                .iter()
                .map(|&condition_index| {
                    let condition =
                        program.rules[target_rule_index].conditions()[condition_index].clone();
                    condition
                })
                .collect();
            let new_args = get_relevant_terms_from_atoms(
                program.rules[target_rule_index].effect(),
                &new_rule_conditions,
            );

            let new_atom = Atom::new(Arguments::new(new_args), aux_predicate, true);
            let new_rule = Rule::new_generic(GenericRule::new(
                new_atom.clone(),
                new_rule_conditions,
                0.0,
                Annotation::None,
                // We can unwrap here because at this point all rules are still
                // directly from schemas
                program.rules[target_rule_index].schema_index().unwrap(),
            ));
            // Need to update the existing rule's variable sources to point to
            // table entries in the new rule.
            program.rules[target_rule_index]
                .variable_source_mut()
                .update_entries_with_new_source(component_index, new_rule.variable_source());
            new_rules.push(new_rule);
            new_conditions.push(new_atom);
        }
    }

    program.rules[target_rule_index].set_condition(new_conditions);
    program.rules.extend(new_rules);

    program
}

/// Returns the connected components of the conditions of a rule, where each
/// component is represented as a vector of condition indices. Two conditions
/// are connected in the graph if they share a variable.
fn get_components(rule: &Rule) -> Vec<Vec<usize>> {
    let mut graph = Graph::new();

    for i in 0..rule.conditions().len() {
        graph.add_node(i);
    }

    for (i, j) in (0..rule.conditions().len()).tuple_combinations() {
        let condition_i = &rule.conditions()[i];
        let condition_j = &rule.conditions()[j];

        if condition_i.shares_variable_with(condition_j) {
            graph.add_edge(i, j);
        }
    }

    graph.get_connected_components()
}

fn get_relevant_terms_from_atoms(effect: &Atom, atoms: &[Atom]) -> Vec<Term> {
    effect
        .arguments()
        .iter()
        .filter_map(|term| {
            if term.is_object() {
                return None;
            }

            if atoms
                .iter()
                .any(|atom| atom.arguments().iter().contains(term))
            {
                Some(term.to_owned())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::{
            datalog::{
                program::Program, transformations::remove_action_predicates, Annotation,
                AnnotationGenerator,
            },
            Task,
        },
        test_utils::*,
    };
    use itertools::Itertools;
    use std::rc::Rc;

    #[test]
    fn test_connected_components_splitting() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), annotation_generator);
        // we normally remove action predicates before splitting into connected
        // components, so we do it here as well
        program = remove_action_predicates(program);

        for rule_index in 0..program.rules.len() {
            program = split_into_connected_components(program, rule_index);
        }

        assert_eq!(
            program.predicate_names,
            vec![
                "clear",
                "on-table",
                "arm-empty",
                "holding",
                "on",
                "applicable-pickup",
                "applicable-putdown",
                "applicable-stack",
                "applicable-unstack",
                "p$9",
                "p$10",
                "p$11",
            ]
        );

        assert_eq!(
            program
                .rules
                .iter()
                .map(|rule| format!("{}", rule))
                .collect_vec(),
            vec![
                // the pickup rule for adding (holding ?ob) gets split into two
                "(3(?0) <- 2(), 9(?0)  | weight: 1; annotation: None; schema_index: 0)",
                // putdown rules, add effects (clear ?ob), (arm-empty),
                // (on-table ?ob), these don't get split
                "(0(?0) <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                "(2() <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                "(1(?0) <- 3(?0)  | weight: 1; annotation: None; schema_index: 1)",
                // stack rules, add effects (arm-empty) (clear ?ob) (on ?ob
                // ?underob), these also don't get split
                "(2() <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                "(0(?0) <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                "(4(?0, ?1) <- 3(?0), 0(?1)  | weight: 1; annotation: None; schema_index: 2)",
                // unstack applicability rule for adding (holding ?ob) and
                // (clear ?underob), both get split
                "(3(?0) <- 2(), 10(?0)  | weight: 1; annotation: None; schema_index: 3)",
                "(0(?1) <- 2(), 11(?1)  | weight: 1; annotation: None; schema_index: 3)",
                // pickup auxillary rule
                "(9(?0) <- 1(?0), 0(?0)  | weight: 0; annotation: None; schema_index: 0)",
                // unstack auxillary rules
                "(10(?0) <- 0(?0), 4(?0, ?1)  | weight: 0; annotation: None; schema_index: 3)",
                "(11(?1) <- 0(?0), 4(?0, ?1)  | weight: 0; annotation: None; schema_index: 3)",
            ]
        )
    }
}
