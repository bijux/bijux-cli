#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Application-layer command dispatch and runtime query interfaces.
pub mod application;
/// Stable shared catalogs and contracts used across dev-cli capabilities.
pub mod catalog;
/// Domain modules organized by maintainer command ownership.
pub mod domain;
/// Reusable technical adapters for filesystem/process/clock concerns.
pub mod infrastructure;
/// Backward-compatibility shim for older `support` imports.
pub mod support;

pub use application::dispatch;
pub use catalog::report_envelope as reporting;
pub use domain::automation_contracts as scripts;
pub use domain::cockpit;
pub use domain::config;
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
pub use domain::route_audit;
pub use domain::routes;
pub use domain::runtime_contracts as contracts;
pub use domain::runtime_identity;
pub use domain::rustdoc;
pub use domain::script_audit;
pub use domain::state_audit;
pub use domain::status;

pub use catalog::command_registry::{
    command_registry, DevCliCommand, DevCliCommandGroup, DevCliCommandMetadata, ReportContext,
    MAINTAINER_COMMAND_NAMESPACE,
};
