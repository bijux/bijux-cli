#![forbid(unsafe_code)]
//! Feature modules containing business behavior and policies.

/// Configuration domain logic and command behavior.
pub mod config;
/// Read-only runtime diagnostics query providers.
pub mod diagnostics;
/// History state management and command behavior.
pub mod history;
/// Installation compatibility and state management behavior.
pub mod install;
/// Memory state management and command behavior.
pub mod memory;
/// Plugin discovery, manifest validation, and registry state.
pub mod plugins;
