#![forbid(unsafe_code)]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

pub mod contracts;
pub mod crate_health;
pub mod docs_audit;
pub mod env;
pub mod package_health;
pub mod parity;
pub mod registry;
pub mod routes;
pub mod runtime_identity;
pub mod script_audit;
pub mod state_audit;
pub mod status;
mod types;

pub use types::{DevCliCommand, ReportContext};
