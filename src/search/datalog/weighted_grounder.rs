use crate::search::{
    datalog::{
        fact::{self, facts_from_state, Fact, FactRegistry},
        program::Program,
        rule_matcher::{self, RuleMatcher},
        rules::Rule,
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

    pub fn ground(&self, program: &Program, state: &DBState) -> f64 {
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
                let rule = &program.rules[rule_match.rule_index.value()];

                match rule {
                    Rule::Project(project_rule) => {
                        todo!()
                    }
                    Rule::Product(product_rule) => {
                        todo!()
                    }
                    Rule::Join(join_rule) => {
                        todo!()
                    }
                    Rule::Generic(_) => {
                        panic!("All rules should be normalized to Project, Product, or Join rules")
                    }
                }
            }
        }

        f64::INFINITY
    }
}
