#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Domain report builders and policy logic for maintainer workflows.
pub mod domain;
/// Interface-layer command dispatch for `bijux dev cli`.
pub mod interface;
/// Shared crate-level contracts and metadata types.
pub mod shared;

pub use domain::cockpit;
pub use domain::config;
pub use domain::contracts;
pub use domain::control_plane;
pub use domain::crate_health;
pub use domain::docs_audit;
pub use domain::env;
pub use domain::evidence;
pub use domain::package_health;
pub use domain::parity;
pub use domain::python;
pub use domain::registry;
pub use domain::release;
pub use domain::repo;
pub use domain::reporting;
pub use domain::route_audit;
pub use domain::routes;
pub use domain::runtime_identity;
pub use domain::rustdoc;
pub use domain::script_audit;
pub use domain::scripts;
pub use domain::state_audit;
pub use domain::status;
pub use interface::dispatch;

pub use shared::types::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
