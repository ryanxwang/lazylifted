use clap::Parser;
use lazylifted::search::{successor_generators::SuccessorGeneratorName, validate, Plan, Task};
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

    let result = validate(&plan, &*generator, &task);
    println!("{:?}", result);
}
