use std::collections::HashSet;

use itertools::Itertools;

use crate::search::datalog::{arguments::Arguments, atom::Atom,
        program::Program,
        rules::{ JoinRule, ProjectRule, Rule, RuleTrait},
        term::Term,
        transformations::{
            connected_components::split_into_connected_components, join_cost::JoinCostType,
        },
        Annotation,
    };

/// Use projection rules to remove constant arguments from the rules.
/// Specifically, given a rule with more than one condition, for each condition
/// with a constant, e.g. `p(a, ?b)`, we add a projection rule
/// ```Datalog
///     aux(?b) <- p(a, ?b)
/// ```
/// and replace the condition `p(a, ?b)` in the original rule with `aux(?b)`.
///
/// This came from the original Powerlifted code. A copy of the original comment
/// is below:
///
/// > This is a quick solution to the problem with constant in join rules.
/// > Ideally we want to either check before the join if the constants match, or
/// > make the rule matcher also take into account the constants.
fn project_away_constant_arguments(mut program: Program) -> Program {
    // TODO-someday: Test this
    let mut new_rules = vec![];
    for rule_index in 0..program.rules.len() {
        for condition_index in 0..program.rules[rule_index].conditions().len() {
            let condition = &program.rules[rule_index].conditions()[condition_index];
            let requires_projection = condition
                .arguments()
                .iter()
                .any(|argument| argument.is_object());
            if !requires_projection {
                continue;
            }
            // we will need to clone anyway, doing it here avoids borrowing
            // issues
            let condition = condition.clone();

            let free_variables: Vec<Term> = condition
                .arguments()
                .iter()
                .filter(|argument| argument.is_variable())
                .cloned()
                .collect();

            let aux_predicate_index = program.new_auxillary_predicate(None);
            let aux_atom = Atom::new(Arguments::new(free_variables), aux_predicate_index, true);
            let project_rule = Rule::new_project(ProjectRule::new(
                aux_atom.clone(),
                condition,
                0.0,
                Annotation::None,
            ));
            new_rules.push(project_rule);
            program.rules[rule_index].update_single_condition(aux_atom, condition_index);
        }
    }

    program.rules.extend(new_rules);
    program
}

/// Check if a rule is a product rule, which is a rule where none of the
/// conditions share any variables.
fn is_product_rule(rule: &Rule) -> bool {
    let mut seen_variables = HashSet::new();
    for condition in rule.conditions() {
        for argument in condition.arguments() {
            if argument.is_object() {
                continue;
            }
            if seen_variables.contains(argument) {
                return false;
            }
            seen_variables.insert(argument);
        }
    }
    true
}

/// To help splitting off some conditions, find all the arguments for the new
/// auxiliary predicate that will be created. This is the intersection of
/// - all the variables in the conditions that will be split off
/// - all the variables in the rest of the rule, including the effect
fn find_arguments_for_split_condition(
    original_rule: &Rule,
    split_condition_indices: &[usize],
) -> Arguments {
    let mut terms_in_new_condition = HashSet::new();
    for &index in split_condition_indices {
        terms_in_new_condition.extend(
            original_rule.conditions()[index]
                .arguments()
                .iter()
                .filter(|argument| argument.is_variable())
                .cloned(),
        );
    }

    let mut remaining_terms = HashSet::new();
    for (condition_index, condition) in original_rule.conditions().iter().enumerate() {
        if split_condition_indices.contains(&condition_index) {
            continue;
        }
        remaining_terms.extend(
            condition
                .arguments()
                .iter()
                .filter(|argument| argument.is_variable())
                .cloned(),
        );
    }
    remaining_terms.extend(
        original_rule
            .effect()
            .arguments()
            .iter()
            .filter(|argument| argument.is_variable())
            .cloned(),
    );

    Arguments::new(
        terms_in_new_condition
            .intersection(&remaining_terms)
            .cloned()
            .sorted()
            .collect(),
    )
}

fn fix_split_rule_source_table(mut split_rule: Rule, join_rules_so_far: &[Rule]) -> Rule {
    // these are the variables with a source so far in the split rule
    let seen_variables: HashSet<usize> = split_rule
        .conditions()
        .iter()
        .flat_map(|condition| condition.variables_set())
        .collect();

    for condition_index in 0..split_rule.conditions().len() {
        for join_rule in join_rules_so_far {
            if join_rule.effect().predicate_index()
                != split_rule.conditions()[condition_index].predicate_index()
            {
                continue;
            }

            let join_rule_variable_source = join_rule.variable_source();
            for table_index in 0..join_rule_variable_source.table().len() {
                let variable_index =
                    join_rule_variable_source.get_variable_index_from_table_index(table_index);
                if seen_variables.contains(&variable_index) {
                    continue;
                }

                // we need to add this variable to the split rule's variable source
                split_rule.variable_source_mut().add_indirect_entry(
                    variable_index,
                    condition_index,
                    table_index,
                );
            }
        }
    }

    split_rule
}

fn split_rule<F>(
    mut rule: Rule,
    condition_indices: Vec<usize>,
    join_rules_so_far: &[Rule],
    aux_predicate_generator: &mut F,
) -> (Rule, Rule)
where
    F: FnMut() -> usize,
{
    assert_eq!(condition_indices.len(), 2);
    let aux_predicate_index = aux_predicate_generator();
    let new_rule_conditions = condition_indices
        .iter()
        .map(|&index| rule.conditions()[index].clone())
        .collect_vec();
    let aux = Atom::new(
        find_arguments_for_split_condition(&rule, &condition_indices),
        aux_predicate_index,
        true,
    );
    let mut split_rule = Rule::new_join(JoinRule::new(
        aux.clone(),
        (
            new_rule_conditions[0].clone(),
            new_rule_conditions[1].clone(),
        ),
        0.0,
        Annotation::None,
    ));

    // We need to make sure the new split rule has the right variable source,
    // since some of the conditions we split off could be auxiliary predicates
    // created by previous join rules, and we need to keep track of variables
    // that were projected away by the previous join rules.
    split_rule = fix_split_rule_source_table(split_rule, join_rules_so_far);

    // We also need to update the original rule to use the new auxiliary predicate
    rule.merge_conditions(&condition_indices, aux, split_rule.variable_source());

    (rule, split_rule)
}

fn convert_to_join_rules<F>(mut rule: Rule, mut aux_predicate_generator: F) -> Vec<Rule>
where
    F: FnMut() -> usize,
{
    let mut join_rules = vec![];
    while rule.conditions().len() > 2 {
        let (index1, index2) = (0..rule.conditions().len())
            .tuple_combinations()
            .min_by_key(|(i, j)| {
                JoinCostType::FastDownward.calculate_join_cost(
                    &rule,
                    &rule.conditions()[*i],
                    &rule.conditions()[*j],
                )
            })
            .unwrap();
        let (existing_rule, new_rule) = split_rule(
            rule,
            vec![index1, index2].into_iter().sorted().collect(),
            &join_rules,
            &mut aux_predicate_generator,
        );
        rule = existing_rule;
        join_rules.push(new_rule);
    }
    // Each iteration should reduce the number of conditions by 1
    assert_eq!(rule.conditions().len(), 2);

    // Now we just convert rule directly to a join rule
    let new_rule = if let Rule::Generic(generic_rule) = rule {
        Rule::Join(generic_rule.to_join_rule())
    } else {
        panic!("Expecting all non-projection rules to be generic at this point of normalisation")
    };
    join_rules.push(new_rule);

    join_rules
}

pub fn convert_rules_to_normal_form(mut program: Program) -> Program {
    for i in 0..program.rules.len() {
        program = split_into_connected_components(program, i);
    }

    program = project_away_constant_arguments(program);

    let mut new_rules = vec![];
    for rule in program.rules {
        if rule.conditions().len() == 1 {
            let new_rule = if let Rule::Generic(generic_rule) = &rule {
                Rule::Project(generic_rule.to_project_rule())
            } else {
                rule
            };
            assert!(new_rule.is_project());
            new_rules.push(new_rule);
        } else if is_product_rule(&rule) {
            let new_rule = if let Rule::Generic(generic_rule) = &rule {
                Rule::Product(generic_rule.to_product_rule())
            } else {
                panic!("Expecting all non-projection rules to be generic at this point of normalisation")
            };
            assert!(new_rule.is_product());
            new_rules.push(new_rule);
        } else {
            new_rules.append(&mut convert_to_join_rules(rule, || {
                // This code is copied from
                // [`Program::new_auxillary_predicate`], we cannot just call it
                // because that would need to mutably borrow the entire program,
                // which we cannot do here.
                let index = program.predicate_names.len();
                let name = format!("p${}", index);
                program.predicate_names.push(name.clone());
                program.predicate_name_to_index.insert(name, index);
                index
            }));
        }
    }

    program.rules = new_rules;
    program
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        search::{Task, datalog::{transformations::remove_action_predicates, AnnotationGenerator}},
        test_utils::*,
    };
    use std::rc::Rc;

    #[test]
    fn project_away_constant_arguments_does_nothing_in_blocksworld() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task, &annotation_generator);
        let original_program = program.clone();
        program = project_away_constant_arguments(program);

        assert_eq!(program, original_program);
    }

    #[test]
    fn test_convert_blocksworld_to_normal_form() {
        let task = Rc::new(Task::from_text(
            BLOCKSWORLD_DOMAIN_TEXT,
            BLOCKSWORLD_PROBLEM13_TEXT,
        ));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task, &annotation_generator);
        program = remove_action_predicates(program);
        program = convert_rules_to_normal_form(program);

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
                "@p9",
                "@p10",
                "@p11",
            ]
        );

        // all the schema indices should be gone since no rule should still be
        // generic
        assert_eq!(
            program
                .rules
                .iter()
                .map(|rule| format!("{}", rule))
                .collect_vec(),
            vec![
                // the pickup rule for adding (holding ?ob) gets split into two
                "(3(?0) <- 2(), 9(?0)  | weight: 1; annotation: None)",
                // putdown rules, add effects (clear ?ob), (arm-empty),
                // (on-table ?ob), these don't get split
                "(0(?0) <- 3(?0)  | weight: 1; annotation: None)",
                "(2() <- 3(?0)  | weight: 1; annotation: None)",
                "(1(?0) <- 3(?0)  | weight: 1; annotation: None)",
                // stack rules, add effects (arm-empty) (clear ?ob) (on ?ob
                // ?underob), these also don't get split
                "(2() <- 3(?0), 0(?1)  | weight: 1; annotation: None)",
                "(0(?0) <- 3(?0), 0(?1)  | weight: 1; annotation: None)",
                "(4(?0, ?1) <- 3(?0), 0(?1)  | weight: 1; annotation: None)",
                // unstack applicability rule for adding (holding ?ob) and
                // (clear ?underob), both get split
                "(3(?0) <- 2(), 10(?0)  | weight: 1; annotation: None)",
                "(0(?1) <- 2(), 11(?1)  | weight: 1; annotation: None)",
                // pickup auxillary rule
                "(9(?0) <- 1(?0), 0(?0)  | weight: 0; annotation: None)",
                // unstack auxillary rules
                "(10(?0) <- 0(?0), 4(?0, ?1)  | weight: 0; annotation: None)",
                "(11(?1) <- 0(?0), 4(?0, ?1)  | weight: 0; annotation: None)",
            ]
        );

        // additionally check that the variable sources of split rules are correct
        assert_eq!(
            program.rules[0].variable_source().to_string(),
            "VariableSource {\n  ?0: Indirect { condition_index: 1, table_index: 0 }\n}"
        );
        assert_eq!(
            program.rules[7].variable_source().to_string(),
            "VariableSource {\n  ?0: Indirect { condition_index: 1, table_index: 0 }\n  ?1: Indirect { condition_index: 1, table_index: 1 }\n}"
        );
        assert_eq!(
            program.rules[8].variable_source().to_string(),
            "VariableSource {\n  ?0: Indirect { condition_index: 1, table_index: 0 }\n  ?1: Indirect { condition_index: 1, table_index: 1 }\n}"
        );
        assert_eq!(
            program.rules[9].variable_source().to_string(),
            "VariableSource {\n  ?0: Direct { condition_index: 0, argument_index: 0 }\n}"
        );
        assert_eq!(
            program.rules[10].variable_source().to_string(),
            "VariableSource {\n  ?0: Direct { condition_index: 0, argument_index: 0 }\n  ?1: Direct { condition_index: 1, argument_index: 1 }\n}"
        );
        assert_eq!(
            program.rules[11].variable_source().to_string(),
            "VariableSource {\n  ?0: Direct { condition_index: 0, argument_index: 0 }\n  ?1: Direct { condition_index: 1, argument_index: 1 }\n}"
        );
    }

    #[test]
    fn test_convert_spanner_to_normal_form() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = remove_action_predicates(program);
        program = convert_rules_to_normal_form(program);

        assert_eq!(
            program.predicate_names,
            vec![
                "at",
                "carrying",
                "usable",
                "link",
                "tightened",
                "loose",
                "applicable-walk",
                "applicable-pickup_spanner",
                "applicable-tighten_nut",
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
                // walk adds (at ?m ?end)
                "(0(?2, ?1) <- 3(?0, ?1), 0(?2, ?0)  | weight: 1; annotation: None)",
                // pickup_spanner adds (carrying ?m ?s)
                "(1(?2, ?1) <- 0(?1, ?0), 0(?2, ?0)  | weight: 1; annotation: None)",
                // tighten_nut adds (tightened ?n), this gets split
                "(9(?0, ?3) <- 5(?3), 0(?3, ?0)  | weight: 0; annotation: None)",
                "(10(?2) <- 2(?1), 1(?2, ?1)  | weight: 0; annotation: None)",
                "(11(?0) <- 0(?2, ?0), 10(?2)  | weight: 0; annotation: None)",
                "(4(?3) <- 9(?0, ?3), 11(?0)  | weight: 1; annotation: None)"
            ]
        );

        // check some interesting variable sources
        assert_eq!(
            program.rules[2].variable_source().to_string(),
            "VariableSource {\n  ?0: Direct { condition_index: 1, argument_index: 1 }\n  ?3: Direct { condition_index: 0, argument_index: 0 }\n}"
        );
        assert_eq!(
            program.rules[3].variable_source().to_string(), 
            "VariableSource {\n  ?1: Direct { condition_index: 0, argument_index: 0 }\n  ?2: Direct { condition_index: 1, argument_index: 0 }\n}"
        );
        assert_eq!(
            program.rules[4].variable_source().to_string(),
            "VariableSource {\n  ?0: Direct { condition_index: 0, argument_index: 1 }\n  ?2: Direct { condition_index: 0, argument_index: 0 }\n  ?1: Indirect { condition_index: 1, table_index: 0 }\n}"
        );
        assert_eq!(
            program.rules[5].variable_source().to_string(),
            "VariableSource {\n  ?0: Indirect { condition_index: 0, table_index: 0 }\n  ?1: Indirect { condition_index: 1, table_index: 2 }\n  ?2: Indirect { condition_index: 1, table_index: 1 }\n  ?3: Indirect { condition_index: 0, table_index: 1 }\n}"
        );
    }

    #[test]
    fn test_convert_satellite_to_normal_form() {
        let mut task = Task::from_text(SATELLITE_DOMAIN_TEXT, SATELLITE_PROBLEM10_TEXT);
        task.remove_negative_preconditions();
        let task = Rc::new(task);
        
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = remove_action_predicates(program);
        program = convert_rules_to_normal_form(program);

        assert_eq!(
            program.predicate_names,
            vec![
                "on_board",
                "supports",
                "pointing",
                "power_avail",
                "power_on",
                "calibrated",
                "have_image",
                "calibration_target",
                "not@pointing",
                "applicable-turn_to",
                "applicable-switch_on",
                "applicable-switch_off",
                "applicable-calibrate",
                "applicable-take_image",
                "p$14",
                "p$15",
                "p$16",
                "p$17",
                "p$18",
            ]
        );

        assert_eq!(
            program
                .rules
                .iter()
                .map(|rule| format!("{}", rule))
                .collect_vec(),
            vec![
                // turn_to adds (pointing ?s ?d_new) and (not@pointing ?s ?d_prev)
                "(2(?0, ?1) <- 8(?0, ?1), 2(?0, ?2)  | weight: 1; annotation: None)",
                "(8(?0, ?2) <- 8(?0, ?1), 2(?0, ?2)  | weight: 1; annotation: None)",
                // switch_on adds (power_on ?i)
                "(4(?0) <- 3(?1), 0(?0, ?1)  | weight: 1; annotation: None)",
                // switch_off adds (power_avail ?s)
                "(3(?1) <- 4(?0), 0(?0, ?1)  | weight: 1; annotation: None)",
                // calibrate gets split
                "(14(?1, ?2) <- 4(?1), 7(?1, ?2)  | weight: 0; annotation: None)",
                "(15(?1, ?2) <- 2(?0, ?2), 0(?1, ?0)  | weight: 0; annotation: None)",
                "(5(?1) <- 14(?1, ?2), 15(?1, ?2)  | weight: 1; annotation: None)",
                // take_image gets split
                "(16(?2) <- 4(?2), 5(?2)  | weight: 0; annotation: None)",
                "(17(?2, ?3) <- 1(?2, ?3), 16(?2)  | weight: 0; annotation: None)",
                "(18(?1, ?2) <- 2(?0, ?1), 0(?2, ?0)  | weight: 0; annotation: None)",
                "(6(?1, ?3) <- 17(?2, ?3), 18(?1, ?2)  | weight: 1; annotation: None)"
            ]
        );
    }
}
