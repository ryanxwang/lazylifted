use crate::search::{
    datalog::{
        atom::Atom,
        fact::{self, facts_from_state, Fact, FactCost, FactRegistry},
        program::Program,
        rule_matcher::{self, RuleMatcher},
        rules::{JoinConditionPosition, JoinRule, ProductRule, ProjectRule, Rule, RuleTrait},
        term::{self, Term},
    },
    DBState,
};
use priority_queue::PriorityQueue;
use std::collections::HashSet;

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
        let mut fact_registry = FactRegistry::new();
        let mut initial_fact_ids = HashSet::new();
        let mut priority_queue = PriorityQueue::new();

        for fact in &program.static_facts {
            let cost = fact.cost();
            let fact_id = fact_registry.add_or_get_fact(fact.clone());
            priority_queue.push(fact_id, cost);
            initial_fact_ids.insert(fact_id);
        }
        for fact in facts_from_state(state, &program.task) {
            let cost = fact.cost();
            let fact_id = fact_registry.add_or_get_fact(fact);
            priority_queue.push(fact_id, cost);
            initial_fact_ids.insert(fact_id);
        }

        while let Some((current_fact_id, current_cost)) = priority_queue.pop() {
            let current_fact = fact_registry.get_by_id(current_fact_id);

            if current_fact.atom().predicate_index() == program.goal_predicate_index.unwrap() {
                // TODO-soon: backchain to execute annotations

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
                        self.project(project_rule, current_fact, &mut new_facts);
                    }
                    #[allow(unused_variables)]
                    Rule::Product(product_rule) => {
                        todo!()
                    }
                    Rule::Join(join_rule) => {
                        self.join(
                            join_rule,
                            current_fact,
                            rule_match.condition_index,
                            &mut new_facts,
                        );
                    }
                    Rule::Generic(_) => {
                        panic!("All rules should be normalised to Project, Product, or Join rules")
                    }
                }
            }
        }

        f64::INFINITY
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

            for term in rule.condition(other_position).arguments() {
                if term.is_object() {
                    continue;
                }
                if let Some(position_in_effect) =
                    rule.variable_position_in_effect().get(term.index())
                {
                    assert!(reached_fact.atom().arguments()[term.index()].is_object());
                    new_arguments[position_in_effect] =
                        reached_fact.atom().arguments()[term.index()];
                }
            }

            let cost = self.aggregate(&[fact.cost(), reached_fact.cost()], rule.weight());
            new_facts.push(Fact::new(
                Atom::new(
                    new_arguments,
                    rule.effect().predicate_index(),
                    rule.effect().is_artificial_predicate(),
                ),
                cost,
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
                Term::Variable(variable_index) => {
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
        ));
    }
}
