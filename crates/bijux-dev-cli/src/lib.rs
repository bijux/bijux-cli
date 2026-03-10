#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

pub mod contracts;
pub mod control_plane;
pub mod crate_health;
pub mod docs_audit;
pub mod env;
pub mod package_health;
pub mod parity;
pub mod registry;
pub mod reporting;
pub mod route_audit;
pub mod routes;
pub mod runtime_identity;
pub mod script_audit;
pub mod scripts;
pub mod state_audit;
pub mod status;
mod types;

pub use types::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
