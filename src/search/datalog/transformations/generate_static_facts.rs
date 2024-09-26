use crate::search::datalog::{
    arguments::Arguments, atom::Atom, fact::Fact, program::Program, term::Term,
};

pub fn generate_static_facts(mut program: Program) -> Program {
    let static_predicates = program.task.static_predicates();

    // Add all the static facts from the initial state to the program.
    for atom in program.task.initial_state.atoms() {
        if !static_predicates.contains(&atom.predicate_index()) {
            continue;
        }

        let terms: Vec<Term> = atom
            .arguments()
            .iter()
            .map(|object_index| Term::new_object(*object_index))
            .collect();
        program.facts.push(Fact::new(
            Atom::new(Arguments::new(terms), atom.predicate_index(), false),
            0.0,
        ));
    }

    program
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::{
        search::{
            datalog::{transformations::remove_action_predicates, Annotation, AnnotationGenerator},
            Task,
        },
        test_utils::*,
    };
    use std::rc::Rc;

    #[test]
    fn test_generate_static_facts_spanner() {
        let task = Rc::new(Task::from_text(SPANNER_DOMAIN_TEXT, SPANNER_PROBLEM10_TEXT));
        let annotation_generator: AnnotationGenerator = Box::new(|_, _| Annotation::None);

        let mut program = Program::new_raw_for_tests(task.clone(), &annotation_generator);
        program = generate_static_facts(program);
        assert_eq!(
            program
                .facts
                .iter()
                .map(|fact| fact.to_string())
                .collect_vec(),
            vec![
                "(fact 3(7, 8), cost 0)",
                "(fact 3(8, 9), cost 0)",
                "(fact 3(9, 10), cost 0)",
                "(fact 3(10, 11), cost 0)",
                "(fact 3(11, 12), cost 0)",
                "(fact 3(12, 13), cost 0)",
                "(fact 3(13, 14), cost 0)"
            ]
        )
    }
}
