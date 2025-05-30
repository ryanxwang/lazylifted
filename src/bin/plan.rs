use clap::{Parser, Subcommand};
use lazylifted::search::{
    heuristics::{PartialActionHeuristicNames, StateHeuristicNames},
    problem_formulations::{PartialActionProblem, StateSpaceProblem},
    search_engines::{SearchEngineName, SearchResult, TerminationCondition},
    successor_generators::SuccessorGeneratorName,
    validate, Task, Verbosity,
};
use pyo3::Python;
use std::{path::PathBuf, rc::Rc, time::Duration};
use tracing::info;

#[derive(Parser)]
#[command(version)]
/// Run the lazylifted planner.
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(help = "The PDDL domain file")]
    domain: PathBuf,
    #[arg(help = "The PDDL problem instance file")]
    problem: PathBuf,
    #[arg(
        help = "The output plan file, defaults to <domain>_<problem>.plan",
        short = 'o',
        long = "output",
        id = "OUTPUT"
    )]
    plan: Option<PathBuf>,
    #[arg(
        value_enum,
        help = "The search engine to use",
        short = 'e',
        long = "engine",
        id = "ENGINE",
        default_value_t = SearchEngineName::GBFS
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
        help = "The saved model (as a path) to use for the heuristic \
        evaluator, only needed for heuristics that require training.",
        short = 'm',
        long = "model",
        id = "MODEL"
    )]
    saved_model: Option<PathBuf>,
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
    // These limits are useful, because even though the user can just enforce
    // these limits with utilities like `ulimit`, we get to provide better
    // messages when the limits are exceeded if we are the ones enforcing them.
    #[arg(
        help = "The time limit for the search, supports syntax like 30min",
        long = "time-limit",
        id = "TIME_LIMIT",
        value_parser = humantime::parse_duration,
    )]
    time_limit: Option<Duration>,
    #[arg(
        help = "The memory limit for the search, in megabytes",
        long = "memory-limit",
        id = "MEMORY_LIMIT"
    )]
    memory_limit_mb: Option<usize>,
}

#[derive(Subcommand)]
#[allow(clippy::enum_variant_names)]
enum Commands {
    /// Run a state space search. This is the traditional search problem where
    /// the search engine explores a state space, transitioning between states
    /// via ground actions.
    StateSpaceSearch {
        #[arg(
            value_enum,
            help = "The heuristic evaluator to use",
            long = "heuristic",
            id = "HEURISTIC"
        )]
        heuristic_name: StateHeuristicNames,
    },
    /// Run a partial space search. This means the search engine explores a
    /// graph of (state, partial action) pairs, transitioning between nodes
    /// via gradually building up the partial action to a full action.
    PartialSpaceSearch {
        #[arg(
            value_enum,
            help = "The heuristic evaluator to use",
            long = "heuristic",
            id = "HEURISTIC"
        )]
        heuristic_name: PartialActionHeuristicNames,
    },
}

fn main() {
    let cli = Cli::parse();

    let level: tracing::Level = cli.verbosity.into();
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(cli.colour)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .compact()
        .init();

    let task = Task::from_path(&cli.domain, &cli.problem);

    // Assume GIL is required
    Python::with_gil(|_| plan(cli, task));
}

fn plan(cli: Cli, mut task: Task) {
    let successor_generator = cli.successor_generator_name.create(&task);
    let termination_condition = TerminationCondition::new(cli.time_limit, cli.memory_limit_mb);

    match cli.command {
        Commands::StateSpaceSearch { heuristic_name } => {
            task.satisfy_requirements(&heuristic_name.requirements());
        }
        Commands::PartialSpaceSearch { heuristic_name } => {
            task.satisfy_requirements(&heuristic_name.requirements());
        }
    }
    let task = Rc::new(task);

    let result = match cli.command {
        Commands::StateSpaceSearch { heuristic_name } => {
            let heuristic = heuristic_name.create(
                task.clone(),
                cli.successor_generator_name,
                cli.saved_model.as_deref(),
            );
            let problem = StateSpaceProblem::new(task.clone(), successor_generator, heuristic);
            cli.search_engine_name
                .search(Box::new(problem), termination_condition)
        }
        Commands::PartialSpaceSearch { heuristic_name } => {
            let heuristic = heuristic_name.create(
                task.clone(),
                cli.successor_generator_name,
                cli.saved_model.as_deref(),
            );
            let problem = PartialActionProblem::new(task.clone(), successor_generator, heuristic);
            cli.search_engine_name
                .search(Box::new(problem), termination_condition)
        }
    };

    match result {
        SearchResult::Success(plan) => {
            info!("validating plan");
            let generator = cli.successor_generator_name.create(&task);
            // TODO-soon add option to use VAL for validation
            let validation_result = validate(&plan, &*generator, &task);
            match validation_result {
                Ok(()) => info!("plan is valid"),
                Err(e) => {
                    info!("plan is invalid: {}", e);
                    return;
                }
            }
            info!("plan found");
            info!(plan_length = plan.len());

            println!("Plan found:");
            println!("{}", plan.to_string(&task));
            println!("Plan length: {}", plan.len());

            let plan_path = match &cli.plan {
                Some(path) => path.clone(),
                None => PathBuf::from(format!(
                    "{}-{}.plan",
                    task.domain_name(),
                    task.problem_name()
                )),
            };

            std::fs::write(plan_path, plan.to_string(&task)).expect("Failed to write plan file");
        }
        _ => {
            info!("no plan found");
            println!("No plan found: {:?}", result);
        }
    }
}
