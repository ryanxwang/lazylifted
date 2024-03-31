use clap::Parser;
use lazylifted::Task;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "DOMAIN")]
    domain: PathBuf,
    #[arg(short, long, value_name = "PROBLEM")]
    problem: PathBuf,
}

fn main() {
    let args = Args::parse();
    let task = Task::from_path(&args.domain, &args.problem);
    println!("{:?}", task);
}
