#![forbid(unsafe_code)]
//! Intentional public facade for runtime and query consumers.

pub mod config;
pub mod diagnostics;
pub mod install;
pub mod kernel;
pub mod output;
pub mod parser;
pub mod plugins;
pub mod repl;
pub mod routing;
pub mod runtime;
pub mod version;
