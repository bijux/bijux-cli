#![forbid(unsafe_code)]
//! Snapshot test for rendered command tree roots.

use bijux_cli_contracts as _;
use bijux_cli_routing::registry::RouteRegistry;
use clap as _;
use proptest as _;
use strsim as _;
use thiserror as _;
use serde as _;
use serde_json as _;

#[test]
fn command_tree_snapshot_matches_expected() {
    let registry = RouteRegistry::default();
    let actual = registry.render_command_tree();
    let expected = include_str!("snapshots/command_tree.txt");
    assert_eq!(actual, expected);
}
