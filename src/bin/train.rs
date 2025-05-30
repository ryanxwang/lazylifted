use clap::Parser;
use lazylifted::{
    learning::models::{ModelConfig, TrainingInstance},
    search::{Plan, Task},
};
use pyo3::Python;
use serde::Deserialize;
use std::{fs, path::PathBuf};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(
        help = "The path to the data config file",
        id = "DATA",
        short = 'd',
        long = "data"
    )]
    data_config: PathBuf,
    #[arg(
        help = "The path to the model config file",
        id = "MODEL",
        short = 'm',
        long = "model"
    )]
    model_config: PathBuf,
    #[arg(
        help = "The path to save the trained model. Two files will be saved:
        <save_path>.pkl and <save_path>.ron - one for Python and one for Rust",
        id = "SAVE",
        short = 's',
        long = "save",
        default_value = "trained"
    )]
    save_path: PathBuf,
    #[arg(help = "Whether to use coloured output", short = 'c', long = "colour")]
    colour: bool,
    #[arg(help = "Verbose output", short = 'v', long = "verbose")]
    verbose: bool,
}

#[derive(Deserialize, Debug)]
struct DataConfig {
    domain_pddl: PathBuf,
    problems_dir: PathBuf,
    plans_dir: PathBuf,
}

fn load_data(data_config: &DataConfig) -> Vec<TrainingInstance> {
    let mut instances = Vec::new();

    let plan_paths: Vec<PathBuf> = fs::read_dir(&data_config.plans_dir)
        .expect("Failed to read plans directory, does it exist?")
        .map(|entry| entry.unwrap().path())
        .collect();

    for plan_path in plan_paths.iter() {
        let problem_name = plan_path.file_stem().unwrap().to_str().unwrap();
        let problem_path = data_config
            .problems_dir
            .join(format!("{}.pddl", problem_name));

        match problem_path.try_exists() {
            Ok(false) | Err(_) => {
                warn!(
                    "Skipping training instance because cannot verify problem file exists: {}",
                    problem_path.display()
                );
                continue;
            }
            Ok(true) => {}
        };

        let task = Task::from_path(&data_config.domain_pddl, &problem_path);
        let plan = Plan::from_path(plan_path, &task);

        instances.push(TrainingInstance::new(plan, task));
    }
    info!("loaded and parsed {} training instances", instances.len());
    instances
}

fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_ansi(args.colour)
        .with_line_number(true)
        .with_writer(std::io::stderr)
        .compact()
        .init();

    if args.verbose {
        info!("Verbose output enabled");
        lazylifted::learning::VERBOSE.set(true).unwrap();
    }

    let data_config: DataConfig = toml::from_str(
        &fs::read_to_string(&args.data_config)
            .expect("Unable to load data config, does the file exist?"),
    )
    .expect("Unable to parse data config, is it valid?");
    let training_data = load_data(&data_config);

    Python::with_gil(|_| {
        // It is hacky that we don't actually use the GIL token. This is because
        // everything that needs it actually just unsafely assumes it is
        // acquired. This is so that we don't have to pass the Python token
        // around everywhere. The catch is that we need to make sure everything
        // is actually wrapped in a `Python::with_gil` block.
        let mut model = ModelConfig::from_path(&args.model_config).trainer_from_config();
        model.train(&training_data);
        model.save(&args.save_path);
    });
}
