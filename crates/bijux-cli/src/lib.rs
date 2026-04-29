#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Core runtime primitives for Rust bijux-cli.

pub mod api;
mod bootstrap;
pub mod contracts;
mod features;
mod infrastructure;
mod interface;
mod kernel;
mod routing;
pub mod sdk;
mod shared;
