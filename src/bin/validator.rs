use clap::Parser;
use lazylifted::search::{
    successor_generators::SuccessorGeneratorName, Plan, SuccessorGenerator, Task,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(help = "The PDDL domain file")]
    domain: PathBuf,
    #[arg(help = "The PDDL problem instance file")]
    problem: PathBuf,
    #[arg(help = "The plan file to validate")]
    plan: PathBuf,
    #[arg(
        value_enum,
        help = "The successor generator to use",
        short = 'g',
        long = "generator",
        id = "GENERATOR",
        default_value_t = SuccessorGeneratorName::FullReducer
    )]
    successor_generator_name: SuccessorGeneratorName,
}

fn main() {
    let cli = Cli::parse();

    let task = Task::from_path(&cli.domain, &cli.problem);
    let generator = cli.successor_generator_name.create(&task);
    let plan = Plan::from_path(&cli.plan, &task);
    validate(&plan, &*generator, &task);
}

fn validate(plan: &Plan, generator: &dyn SuccessorGenerator, task: &Task) {
    let mut cur_state = task.initial_state.clone();
    for action in plan.steps() {
        let action_schema = &task.action_schemas()[action.index];
        let applicable_actions = generator.get_applicable_actions(&cur_state, action_schema);

        if !applicable_actions.contains(action) {
            panic!(
                "Action {} is not applicable in state {:?}",
                action.to_string(task),
                cur_state
            );
        }

        cur_state = generator.generate_successor(&cur_state, action_schema, action);
    }

    if !task.goal.is_satisfied(&cur_state) {
        panic!(
            "Plan does not reach goal state, final state is: {:?}",
            cur_state
        );
    }
}
