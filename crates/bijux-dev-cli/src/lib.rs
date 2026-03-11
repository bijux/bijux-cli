#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

pub mod cockpit;
pub mod config;
pub mod contracts;
pub mod control_plane;
pub mod crate_health;
pub mod dispatch;
pub mod docs_audit;
pub mod env;
pub mod evidence;
pub mod package_health;
pub mod parity;
pub mod python;
pub mod registry;
pub mod release;
pub mod repo;
pub mod reporting;
pub mod route_audit;
pub mod routes;
pub mod runtime_identity;
pub mod rustdoc;
pub mod script_audit;
pub mod scripts;
pub mod state_audit;
pub mod status;
pub mod status_script_ids;
mod types;

pub use types::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
