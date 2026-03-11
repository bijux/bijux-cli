#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Application building blocks for argument parsing, workspace resolution, and routing.
pub mod app;
/// Contract inventories and execution boundaries.
#[path = "contracts/mod.rs"]
pub mod contract_engine;
/// Application-layer command dispatch and runtime query interfaces.
pub mod dispatch;
/// Reusable technical adapters for filesystem/process/clock concerns.
pub mod infrastructure;
/// Shared platform contracts and schemas used across dev-cli capabilities.
pub mod platform;
/// Maintainer-facing report modules.
pub mod reports;
/// Status contract inventory and execution services.
pub mod status_contracts;

pub use contract_engine::maintenance;
pub use platform::report_envelope as reporting;
pub use reports::cockpit;
pub use reports::config;
pub use reports::control_plane;
pub use reports::evidence;
pub use reports::python;
pub use reports::release;
pub use reports::repository_health::{
    crate_health, docs_audit, maintenance_audit, package_health, repo, state_audit, status,
};
pub use reports::runtime_surface::{
    contracts, env, parity, registry, route_audit, routes, runtime_identity,
};
pub use reports::rustdoc;

pub use platform::command_registry::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
