use clap::Parser;
use lazylifted::search::{
    heuristics::HeuristicName,
    preferred_operator::PreferredOperatorName,
    search_engines::{SearchEngineName, SearchResult},
    successor_generators::SuccessorGeneratorName,
    Task, Verbosity,
};
use pyo3::Python;
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
        help = "The saved model (as a path) to use for the heuristic \
        evaluator, only needed for heuristics that require training.",
        short = 'm',
        long = "model",
        id = "MODEL"
    )]
    saved_model: Option<PathBuf>,
    #[arg(
        help = "The saved model (as a path) to use for the preferred operator \
        provider. Only meaning if the search engine uses it. Will use the
        WL-ALSG ranker to compute preferred operators so the model should be \
        for WL-ALSG.",
        short = 'p',
        long = "preferred",
        id = "PREFERRED"
    )]
    preferred_operator_model: Option<PathBuf>,
    #[arg(
        value_enum,
        help = "The verbosity level",
        short = 'v',
        long = "verbosity",
        id = "VERBOSITY",
        default_value_t = Verbosity::Normal
    )]
    verbosity: Verbosity,
    #[arg(help = "Whether to use coloured output", short = 'c', long = "colour")]
    colour: bool,
}

fn main() {
    let args = Args::parse();

    let level: tracing::Level = args.verbosity.into();
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(args.colour)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let task = Task::from_path(&args.domain, &args.problem);

    // Assume GIL is required
    Python::with_gil(|_| plan(args, &task));
}

fn plan(args: Args, task: &Task) {
    let successor_generator = args.successor_generator_name.create(task);
    let heuristic = args.heuristic_name.create(&args.saved_model);
    let mut search_engine = args.search_engine_name.create();
    let preferred_operator = args
        .preferred_operator_model
        .map(|model| PreferredOperatorName::WLPALG.create(&model));

    let (result, mut statistics) =
        search_engine.search(task, successor_generator, heuristic, preferred_operator);
    statistics.finalise_search();
    match result {
        SearchResult::Success(plan) => {
            println!("Plan found:");
            for action in &plan {
                println!("{}", action.to_string(task));
            }
            println!("Plan length: {}", plan.len());
        }
        _ => {
            println!("No plan found: {:?}", result);
        }
    }
}
