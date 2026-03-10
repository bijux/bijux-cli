#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Core runtime primitives for Rust bijux-cli.

pub mod bootstrap;
pub mod features;
pub mod infrastructure;
pub mod interface;
pub mod kernel;
pub mod routing;
pub mod shared;
