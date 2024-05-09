mod repl_command;

use clap::Parser;
use dialoguer::{theme::ColorfulTheme, BasicHistory, Input};
use lazylifted::learning::models::{Evaluate, PartialActionModel};
use ordered_float::OrderedFloat;
use pyo3::Python;
use repl_command::ReplCommand;
use std::{collections::HashMap, path::PathBuf};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(help = "The model path")]
    model: PathBuf,
}

fn main() {
    let args = Cli::parse();

    Python::with_gil(|_| {
        let py = unsafe { Python::assume_gil_acquired() };
        let model = PartialActionModel::load(py, &args.model);
        run_repl(model);
    });
}

fn run_repl(model: PartialActionModel) {
    let mut history = BasicHistory::new().max_entries(100).no_duplicates(true);
    let weights = model.get_weights();

    let mut weight_groups = HashMap::new();
    for (colour, weight) in weights.iter().enumerate() {
        let weight = OrderedFloat(*weight);
        let group = weight_groups.entry(weight).or_insert_with(Vec::new);
        group.push(colour);
    }
    let mut weight_groups = weight_groups.into_iter().collect::<Vec<_>>();
    weight_groups
        .sort_by_key(|(weight, colours)| OrderedFloat(-weight.abs() * colours.len() as f64));

    #[allow(clippy::while_let_loop)]
    loop {
        if let Ok(cmd) = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter command")
            .history_with(&mut history)
            .interact_text()
        {
            let cmd = ReplCommand::parse(&cmd);
            match cmd {
                Some(ReplCommand::Exit) => break,
                Some(ReplCommand::Help) => {
                    println!("Commands:");
                    println!("  exit: exit the REPL");
                    println!("  help: show this help message");
                    println!("  get_weight <colour>: get the weight for a colour");
                    println!(
                        "  list_by_weight <num>: list the <num> colours with the highest weight by absolute value"
                    );
                    println!("  get_neighbourhood <colour>: get the neighbourhood for a colour")
                }
                Some(ReplCommand::GetWeight(colour)) => {
                    let weight = weights.get(colour as usize);
                    match weight {
                        Some(weight) => println!("{}", weight),
                        None => println!("Unknown colour"),
                    }
                }
                Some(ReplCommand::ListByWeight(num_to_print)) => {
                    for (weight, colours) in weight_groups.iter().take(num_to_print) {
                        println!("{:?}: {}", colours, weight * colours.len() as f64);
                    }
                }
                Some(ReplCommand::GetNeighbourhood(colour)) => match model.inspect_colour(colour) {
                    Some(neighbourhood) => {
                        println!("Neighbourhood for colour {}: {:?}", colour, neighbourhood);
                    }
                    None => println!("Unknown colour"),
                },
                None => println!("Unknown command"),
            }
        } else {
            break;
        }
    }
}
