use crate::search::{
    datalog::{
        program::Program, transformations::connected_components::split_into_connected_components,
    },
    Task,
};

pub fn convert_rules_to_normal_form(mut program: Program) -> Program {
    for i in 0..program.rules.len() {
        program = split_into_connected_components(program, i);
    }

    todo!()
}
