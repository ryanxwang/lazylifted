mod data_generators;
pub mod graphs;
mod ml;
pub mod models;
mod wl;

use once_cell::sync::OnceCell;

/// Whether to print verbose output, avoids the need to pass a `verbose` flag
/// around
pub static VERBOSE: OnceCell<bool> = OnceCell::new();
