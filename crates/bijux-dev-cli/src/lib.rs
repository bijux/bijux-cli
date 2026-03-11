#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Application building blocks for argument parsing, workspace resolution, and routing.
pub mod app;
/// Application-layer command dispatch and runtime query interfaces.
pub mod dispatch;
/// Shared platform contracts and schemas used across dev-cli capabilities.
pub mod platform;
/// Maintainer-facing report modules organized by business domains.
pub mod domains;
/// Contract inventories and execution boundaries.
#[path = "contracts/mod.rs"]
pub mod contract_engine;
/// Reusable technical adapters for filesystem/process/clock concerns.
pub mod infrastructure;
/// Status contract inventory and execution services.
pub mod status_contracts;

pub use platform::report_envelope as reporting;
pub use domains::cockpit;
pub use domains::config;
pub use domains::control_plane;
pub use domains::evidence;
pub use domains::python;
pub use domains::release;
pub use domains::repository_health::{
    crate_health, docs_audit, maintenance_audit, package_health, repo, state_audit, status,
};
pub use domains::runtime_surface::{
    contracts, env, parity, registry, route_audit, routes, runtime_identity,
};
pub use domains::rustdoc;
pub use contract_engine::maintenance as maintenance;

pub use platform::command_registry::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
