#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Core runtime primitives for Rust bijux-cli.

pub mod app;
mod argv;
mod config;
pub mod entrypoint;
pub mod kernel;
pub mod output;
pub mod plugin;
pub mod query;
pub mod repl;
