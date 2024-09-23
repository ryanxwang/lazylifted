use crate::search::datalog::{
    program::Program,
    rules::{GenericRule, Rule},
};

pub fn remove_action_predicates(program: Program) -> Program {
    let mut new_rules = vec![];

    for applicability_rule in program
        .rules
        .iter()
        // only the applicability rules have articificial predicates
        .filter(|rule| rule.effect().is_artificial_predicate())
    {
        let applicability_predicate = applicability_rule.effect().predicate_index();
        for effect_rule in program.rules.iter().filter(|rule| {
            rule.conditions().len() == 1
                && rule.conditions()[0].predicate_index() == applicability_predicate
        }) {
            new_rules.push(Rule::new_generic(GenericRule::new(
                effect_rule.effect().clone(),
                applicability_rule.conditions().to_owned(),
                applicability_rule.weight() + effect_rule.weight(),
                applicability_rule.annotation().clone(),
                applicability_rule.schema_index().unwrap(),
            )))
        }
    }

    Program {
        rules: new_rules,
        ..program
    }
}
