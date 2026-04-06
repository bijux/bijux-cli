#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux-dev-cli ...` workflows.

#[path = "maintainer/cli/mod.rs"]
pub mod cli;
#[path = "maintainer/contracts/mod.rs"]
pub mod contracts;
#[path = "maintainer/infra/mod.rs"]
pub mod infra;
#[path = "maintainer/reports/mod.rs"]
pub mod reports;
#[path = "maintainer/runtime/mod.rs"]
pub mod runtime;
#[path = "maintainer/schema/mod.rs"]
pub mod schema;
#[path = "maintainer/suites/mod.rs"]
pub mod suites;
