use crate::search::{
    datalog::{
        achiever::Achiever,
        atom::Atom,
        fact::{facts_from_state, Fact, FactCost, FactId},
        program::Program,
        rule_matcher::RuleMatcher,
        rules::{JoinConditionPosition, JoinRule, ProductRule, ProjectRule, Rule, RuleTrait},
        term::Term,
    },
    DBState,
};
use itertools::Itertools;
use priority_queue::PriorityQueue;
use std::{
    cmp::Reverse,
    collections::{HashSet, VecDeque},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatalogHeuristicType {
    Hadd,
    Hmax,
    Hff,
}

#[derive(Debug, Clone)]
pub struct WeightedGrounderConfig {
    pub heuristic_type: DatalogHeuristicType,
}

#[derive(Debug)]
pub struct WeightedGrounder {
    config: WeightedGrounderConfig,
    rule_matcher: RuleMatcher,
}

impl WeightedGrounder {
    pub fn new(program: &Program, config: WeightedGrounderConfig) -> Self {
        let rule_matcher = RuleMatcher::new(&program.rules);

        Self {
            config,
            rule_matcher,
        }
    }

    fn aggregate(&self, fact_costs: &[FactCost], rule_cost: f64) -> FactCost {
        match self.config.heuristic_type {
            DatalogHeuristicType::Hadd | DatalogHeuristicType::Hff => {
                fact_costs.iter().sum::<FactCost>() + rule_cost
            }
            DatalogHeuristicType::Hmax => {
                FactCost::from(fact_costs.iter().max().unwrap() + rule_cost)
            }
        }
    }

    pub fn ground(&self, program: &mut Program, state: &DBState) -> f64 {
        // IMPORTANT: If/when we incorporate action costs, it's important to pay
        // attention to zero cost actions, see Augusto's AAAI 2022 paper.
        let mut initial_fact_ids = HashSet::new();
        let mut priority_queue = PriorityQueue::new();

        for fact in &program.static_facts {
            let cost = fact.cost();
            let fact_id = program.fact_registry.add_or_get_fact(fact.clone());
            priority_queue.push(fact_id, Reverse(cost));
            initial_fact_ids.insert(fact_id);
        }
        for fact in facts_from_state(state, &program.task) {
            let cost = fact.cost();
            let fact_id = program.fact_registry.add_or_get_fact(fact);
            priority_queue.push(fact_id, Reverse(cost));
            initial_fact_ids.insert(fact_id);
        }

        while let Some((current_fact_id, current_cost)) = priority_queue.pop() {
            let current_cost = current_cost.0;
            let current_fact = program.fact_registry.get_by_id(current_fact_id).clone();

            if current_fact.atom().predicate_index() == program.goal_predicate_index.unwrap() {
                Self::backchain_from_goal(&current_fact, &initial_fact_ids, program);
                return current_cost.into();
            }
            if current_fact.cost() < current_cost {
                // this means we've already processed this fact before
                continue;
            }

            for rule_match in self
                .rule_matcher
                .get_matched_rules(current_fact.atom().predicate_index())
            {
                let rule = &mut program.rules[rule_match.rule_index.value()];

                let mut new_facts = vec![];
                match rule {
                    Rule::Project(project_rule) => {
                        assert_eq!(rule_match.condition_index, 0);
                        self.project(project_rule, &current_fact, &mut new_facts);
                    }
                    Rule::Product(product_rule) => {
                        self.product(
                            product_rule,
                            &current_fact,
                            rule_match.condition_index,
                            &mut new_facts,
                        );
                    }
                    Rule::Join(join_rule) => {
                        self.join(
                            join_rule,
                            &current_fact,
                            rule_match.condition_index,
                            &mut new_facts,
                        );
                    }
                    Rule::Generic(_) => {
                        panic!("All rules should be normalised to Project, Product, or Join rules")
                    }
                }

                for new_fact in new_facts {
                    match program.fact_registry.get_id(&new_fact) {
                        Some(existing_fact_id) => {
                            let existing_fact = program.fact_registry.get_by_id(existing_fact_id);
                            if new_fact.cost() < existing_fact.cost() {
                                let cost = new_fact.cost();
                                program
                                    .fact_registry
                                    .replace_at_id(existing_fact_id, new_fact);
                                priority_queue.push(existing_fact_id, Reverse(cost));
                            }
                        }
                        None => {
                            let cost = new_fact.cost();
                            let new_fact_id = program.fact_registry.add_or_get_fact(new_fact);
                            priority_queue.push(new_fact_id, Reverse(cost));
                        }
                    }
                }
            }
        }

        f64::INFINITY
    }

    fn backchain_from_goal(
        goal_fact: &Fact,
        initial_fact_ids: &HashSet<FactId>,
        program: &Program,
    ) {
        let mut seen_fact_ids = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(goal_fact.id());

        while let Some(fact_id) = queue.pop_front() {
            if seen_fact_ids.contains(&fact_id) {
                continue;
            }
            seen_fact_ids.insert(fact_id);
            if initial_fact_ids.contains(&fact_id) {
                continue;
            }

            let fact = program.fact_registry.get_by_id(fact_id);
            let achiever = fact
                .achiever()
                .expect("All achieved, non-initial facts should have an achiever");
            program.rules[achiever.rule_index().value()]
                .annotation()
                .execute(fact_id, program);
            for achieving_fact_id in achiever.rule_body() {
                queue.push_back(*achieving_fact_id);
            }
        }
    }

    fn join(
        &self,
        rule: &mut JoinRule,
        fact: &Fact,
        fact_index_in_condition: usize,
        new_facts: &mut Vec<Fact>,
    ) {
        let join_condition_position = JoinConditionPosition::try_from(fact_index_in_condition)
            .expect(
            "The fact index in the condition should be a valid JoinConditionPosition, i.e. 0 or 1",
        );
        let joining_variable_values = rule
            .joining_variable_positions(join_condition_position)
            .iter()
            .map(|&variable_position| {
                let term = fact.atom().arguments()[variable_position];
                assert!(term.is_object());
                term
            })
            .collect::<Vec<_>>();

        rule.register_reached_fact_for_joining_variables(
            join_condition_position,
            fact.clone(),
            joining_variable_values.clone(),
        );

        // This arguments vector has all the terms that are fixed from the fact, and
        // the rest are variables
        let mut common_new_arguments = rule.effect().arguments().clone();
        for (i, term) in rule
            .condition(join_condition_position)
            .arguments()
            .iter()
            .enumerate()
        {
            if term.is_object() {
                continue;
            }
            if let Some(position_in_effect) = rule.variable_position_in_effect().get(term.index()) {
                assert!(fact.atom().arguments()[i].is_object());
                common_new_arguments[position_in_effect] = fact.atom().arguments()[i];
            }
        }

        let other_position = join_condition_position.other();
        for reached_fact in
            rule.reached_facts_for_joining_variables(other_position, &joining_variable_values)
        {
            // reached_fact should align with common_new_arguments on the already
            // assigned values
            let mut new_arguments = common_new_arguments.clone();

            for (i, term) in rule
                .condition(other_position)
                .arguments()
                .iter()
                .enumerate()
            {
                if term.is_object() {
                    continue;
                }
                if let Some(position_in_effect) =
                    rule.variable_position_in_effect().get(term.index())
                {
                    assert!(reached_fact.atom().arguments()[i].is_object());
                    new_arguments[position_in_effect] = reached_fact.atom().arguments()[i];
                }
            }

            let achiever_body = match join_condition_position {
                JoinConditionPosition::First => {
                    vec![fact.id(), reached_fact.id()]
                }
                JoinConditionPosition::Second => {
                    vec![reached_fact.id(), fact.id()]
                }
            };

            let cost = self.aggregate(&[fact.cost(), reached_fact.cost()], rule.weight());
            new_facts.push(Fact::new(
                Atom::new(
                    new_arguments,
                    rule.effect().predicate_index(),
                    rule.effect().is_artificial_predicate(),
                ),
                cost,
                Some(Achiever::new(rule.index(), achiever_body)),
            ));
        }
    }

    fn project(&self, rule: &ProjectRule, fact: &Fact, new_facts: &mut Vec<Fact>) {
        let mut effect_arguments = rule.effect().arguments().clone();

        for (i, term) in rule.conditions()[0].arguments().iter().enumerate() {
            match term {
                Term::Object(_) => {
                    // check that it matches the fact
                    if fact.atom().arguments()[i] != *term {
                        return;
                    }
                }
                Term::Variable {
                    variable_index,
                    type_index: _,
                } => {
                    let position_in_effect =
                        rule.variable_position_in_effect().get(*variable_index);
                    if let Some(position_in_effect) = position_in_effect {
                        effect_arguments[position_in_effect] = fact.atom().arguments()[i];
                    }
                }
            }
        }

        new_facts.push(Fact::new(
            Atom::new(
                effect_arguments,
                rule.effect().predicate_index(),
                rule.effect().is_artificial_predicate(),
            ),
            fact.cost() + rule.weight(),
            Some(Achiever::new(rule.index(), vec![fact.id()])),
        ));
    }

    fn product(
        &self,
        rule: &mut ProductRule,
        fact: &Fact,
        fact_index_in_condition: usize,
        new_facts: &mut Vec<Fact>,
    ) {
        // In powerlifted, there are comments around this function that says
        // that for product rules, there are only two scenarios:
        // 1. The rule effect is ground
        // 2. Every free variable in the body is also in the effect
        //
        // I (rywang) am not entirely convinced. More importantly, I don't think
        // we should depend on this assumption, even if it were true. Instead,
        // We compute the cartesian product of all the reached facts for each
        // condition (where for condition at fact_index_in_condition, we only
        // consider the given fact), and instantiate the effect with this.

        // verify that ground objects in the condition match the fact
        for (i, term) in rule.conditions()[fact_index_in_condition]
            .arguments()
            .iter()
            .enumerate()
        {
            if term.is_object() && fact.atom().arguments()[i] != *term {
                return;
            }
        }

        rule.add_reached_fact(fact_index_in_condition, fact.clone());

        for instantiation in (0..rule.conditions().len())
            .map(|i| {
                if i == fact_index_in_condition {
                    vec![fact.clone()]
                } else {
                    rule.reached_facts(i).to_vec()
                }
            })
            .multi_cartesian_product()
        {
            let mut effect_arguments = rule.effect().arguments().clone();
            for (condition_index, fact) in instantiation.iter().enumerate() {
                for (i, term) in rule.conditions()[condition_index]
                    .arguments()
                    .iter()
                    .enumerate()
                {
                    if term.is_object() {
                        continue;
                    }
                    if let Some(position_in_effect) =
                        rule.variable_position_in_effect().get(term.index())
                    {
                        effect_arguments[position_in_effect] = fact.atom().arguments()[i];
                    }
                }
            }

            new_facts.push(Fact::new(
                Atom::new(
                    effect_arguments,
                    rule.effect().predicate_index(),
                    rule.effect().is_artificial_predicate(),
                ),
                self.aggregate(
                    &instantiation
                        .iter()
                        .map(|fact| fact.cost())
                        .collect::<Vec<_>>(),
                    rule.weight(),
                ),
                Some(Achiever::new(
                    rule.index(),
                    instantiation.iter().map(|fact| fact.id()).collect(),
                )),
            ));
        }
    }
}
