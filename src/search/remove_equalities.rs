use crate::parsed_types::{
    Atom, Domain, Literal, PredicateDefinition, Problem, PropCondition, Typed, TypedList, Variable,
};

const OBJECT_EQUALITY_PREDICATE_NAME: &str = "@object-equal";

/// Remove object equalities in action preconditions by adding a new auxiliary
/// predicate for object equality, and adding new atoms for it to the initial
/// state and replacing object equalities in action preconditions with atoms of
/// the new auxiliary predicate. This function is a no-op if there are no object
/// equalities in the domain.
pub fn remove_equalities(mut domain: Domain, mut problem: Problem) -> (Domain, Problem) {
    if !equalities_exist(&domain) {
        return (domain, problem);
    }

    domain.add_predicate(PredicateDefinition::new(
        OBJECT_EQUALITY_PREDICATE_NAME.into(),
        TypedList::new(vec![
            Typed::new_object(Variable::from_str("?x")),
            Typed::new_object(Variable::from_str("?y")),
        ]),
    ));

    let mut new_literals = Vec::new();
    for object in problem.objects().iter() {
        new_literals.push(Literal::new(Atom::new(
            OBJECT_EQUALITY_PREDICATE_NAME.into(),
            vec![object.value().clone(), object.value().clone()],
        )));
    }
    for literal in new_literals {
        problem.add_init(literal);
    }

    for action in domain.actions_mut() {
        for precondition in action.preconditions_mut() {
            *precondition = update_precondition(precondition);
        }
    }

    (domain, problem)
}

fn equalities_exist(domain: &Domain) -> bool {
    domain
        .actions()
        .iter()
        .any(|action| action.preconditions().iter().any(has_equality))
}

fn has_equality(cond: &PropCondition) -> bool {
    match cond {
        PropCondition::Equality(_, _) => true,
        PropCondition::Not(cond) => has_equality(cond),
        PropCondition::And(conds) => conds.iter().any(has_equality),
        PropCondition::Or(conds) => conds.iter().any(has_equality),
        PropCondition::Imply(a, b) => has_equality(a) || has_equality(b),
        _ => false,
    }
}

fn update_precondition(cond: &PropCondition) -> PropCondition {
    match cond {
        PropCondition::Equality(a, b) => PropCondition::new_atom(Atom::new(
            OBJECT_EQUALITY_PREDICATE_NAME.into(),
            vec![a.clone(), b.clone()],
        )),
        PropCondition::Not(cond) => PropCondition::new_not(update_precondition(cond)),
        PropCondition::And(conds) => PropCondition::new_and(conds.iter().map(update_precondition)),
        PropCondition::Or(conds) => PropCondition::new_or(conds.iter().map(update_precondition)),
        PropCondition::Imply(a, b) => {
            PropCondition::new_imply(update_precondition(a), update_precondition(b))
        }
        _ => cond.clone(),
    }
}
