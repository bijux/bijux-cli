#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Core runtime primitives for Rust bijux-cli.

pub mod app;
pub mod bootstrap;
mod argv;
pub mod cli;
mod config;
pub mod entrypoint;
pub mod interface;
pub mod install;
pub mod kernel;
pub mod output;
pub mod plugin;
pub mod query;
pub mod repl;
pub mod routing;
