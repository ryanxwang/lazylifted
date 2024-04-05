use clap;
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Silent,
    Normal,
    Verbose,
    Debug,
}

impl From<Verbosity> for tracing::Level {
    fn from(value: Verbosity) -> Self {
        match value {
            Verbosity::Silent => tracing::Level::ERROR,
            Verbosity::Normal => tracing::Level::INFO,
            Verbosity::Verbose => tracing::Level::DEBUG,
            Verbosity::Debug => tracing::Level::TRACE,
        }
    }
}
