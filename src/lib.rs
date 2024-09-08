#![warn(missing_debug_implementations)]
#![deny(dead_code)]
#![deny(non_ascii_idents)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unit_bindings)]
#![warn(unused_crate_dependencies)]
#![warn(unused_qualifications)]

// Crate dependencies used in binary/test but not in library. Unfortunately
// cargo does not yet allow specifying dependencies for binaries only.
use assert_approx_eq as _;
use console as _;
use dialoguer as _;
use humantime as _;
use tracing_subscriber as _;

pub mod learning;
pub mod parsed_types;
pub mod parsers;
pub mod search;

#[cfg(test)]
mod test_utils;
