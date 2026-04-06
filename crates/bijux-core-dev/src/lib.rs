#![forbid(unsafe_code)]
#![recursion_limit = "512"]
//! Maintainer control-plane modules for `bijux-dev-cli ...` workflows.

#[path = "../dev-cli/src/cli/mod.rs"]
pub mod cli;
#[path = "../dev-cli/src/contracts/mod.rs"]
pub mod contracts;
#[path = "../dev-cli/src/infra/mod.rs"]
pub mod infra;
#[path = "../dev-cli/src/reports/mod.rs"]
pub mod reports;
#[path = "../dev-cli/src/runtime/mod.rs"]
pub mod runtime;
#[path = "../dev-cli/src/schema/mod.rs"]
pub mod schema;
#[path = "../dev-cli/src/suites/mod.rs"]
pub mod suites;
