#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux dev cli ...` workflows.
//!
//! This crate is intentionally focused on maintainer-facing report assembly and
//! control-plane orchestration. Runtime command law remains in runtime crates.

/// Command-line routing, workspace resolution, and dispatch contracts.
pub mod cli;
/// Contract inventories and execution boundaries.
pub mod contracts;
/// Reusable technical adapters for filesystem/process/clock concerns.
pub mod infra;
/// Maintainer-facing report modules.
pub mod reports;
/// Runtime-query bridge and process entrypoint for delegated execution.
pub mod runtime;
/// Shared schemas used across dev-cli capabilities.
pub mod schema;
/// Contract execution suites grouped by control-plane domain.
pub mod suites;
