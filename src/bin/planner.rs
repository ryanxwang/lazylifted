use clap::Parser;
use lazylifted::search::{
    heuristics::HeuristicName,
    search_engines::{SearchEngine, SearchEngineName, SearchResult},
    successor_generators::SuccessorGeneratorName,
    Task, Verbosity,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version)]
/// Run the lazylifted planner.
struct Args {
    #[arg(help = "The PDDL domain file")]
    domain: PathBuf,
    #[arg(help = "The PDDL problem instance file")]
    problem: PathBuf,
    #[arg(
        value_enum,
        help = "The search algorithm to use",
        short = 's',
        long = "search",
        id = "SEARCH"
    )]
    search_engine_name: SearchEngineName,
    #[arg(
        value_enum,
        help = "The successor generator to use",
        short = 'g',
        long = "generator",
        id = "GENERATOR",
        default_value_t = SuccessorGeneratorName::FullReducer
    )]
    successor_generator_name: SuccessorGeneratorName,
    #[arg(
        value_enum,
        help = "The heuristic evaluator to use",
        short = 'e',
        long = "evaluator",
        id = "EVLUATOR"
    )]
    heuristic_name: HeuristicName,
    #[arg(
        value_enum,
        help = "The verbosity level",
        short = 'v',
        long = "verbosity",
        id = "VERBOSITY",
        default_value_t = Verbosity::Normal
    )]
    verbosity: Verbosity,
}

fn main() {
    let args = Args::parse();

    let level: tracing::Level = args.verbosity.into();
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(true)
        .with_line_number(true)
        .compact()
        .init();

    let task = Task::from_path(&args.domain, &args.problem);
    let successor_generator = args.successor_generator_name.create(&task);
    let heuristic = args.heuristic_name.create();
    let mut search_engine = args.search_engine_name.create();

    let (search_result, _statistics) = search_engine.search(&task, successor_generator, &heuristic);
    match search_result {
        SearchResult::Success(plan) => {
            println!("Plan found:");
            for action in &plan {
                println!("{}", action.to_string(&task));
            }
            println!("Plan length: {}", plan.len());
        }
        _ => {
            println!("No plan found: {:?}", search_result);
        }
    }
}
