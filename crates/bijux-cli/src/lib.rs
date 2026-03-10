#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Core runtime primitives for Rust bijux-cli.

pub mod app;
mod argv;
pub mod bootstrap;
pub mod cli;
pub mod entrypoint;
pub mod features;
pub mod infrastructure;
pub mod install;
pub mod interface;
pub mod kernel;
pub mod output;
pub mod plugin;
pub mod query;
pub mod repl;
pub mod routing;
pub mod shared;
