use clap;
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Silent,
    Normal,
    Verbose,
    Debug,
}
