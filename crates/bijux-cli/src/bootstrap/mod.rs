#![forbid(unsafe_code)]
//! Process bootstrap and dependency wiring.

pub mod run;
pub mod wiring;

pub use run::run_cli_from_env;
