#![forbid(unsafe_code)]
//! Routing surface tests relocated under bijux-cli.

#[path = "routing/command_tree_diff_report.rs"]
mod command_tree_diff_report;

#[path = "routing/command_tree_snapshot.rs"]
mod command_tree_snapshot;

#[path = "routing/config_domain_types.rs"]
mod config_domain_types;

#[path = "routing/dev_cli_inventory_boundaries.rs"]
mod dev_cli_inventory_boundaries;

#[path = "routing/dev_cli_routing_identity_boundaries.rs"]
mod dev_cli_routing_identity_boundaries;

#[path = "routing/envelope_compatibility.rs"]
mod envelope_compatibility;

#[path = "routing/flag_normalization_property.rs"]
mod flag_normalization_property;

#[path = "routing/legacy_forms_regression.rs"]
mod legacy_forms_regression;

#[path = "routing/normalization_property.rs"]
mod normalization_property;

#[path = "routing/parser_abuse.rs"]
mod parser_abuse;

#[path = "routing/parser_fixtures.rs"]
mod parser_fixtures;

#[path = "routing/parser_fuzz.rs"]
mod parser_fuzz;

#[path = "routing/parser_fuzz_regressions.rs"]
mod parser_fuzz_regressions;

#[path = "routing/parser_fuzz_targets.rs"]
mod parser_fuzz_targets;

#[path = "routing/parser_intent.rs"]
mod parser_intent;

#[path = "routing/query_interfaces.rs"]
mod query_interfaces;

#[path = "routing/registry_namespace_policy.rs"]
mod registry_namespace_policy;

#[path = "routing/registry_resolution.rs"]
mod registry_resolution;

#[path = "routing/route_fuzz_regressions.rs"]
mod route_fuzz_regressions;

#[path = "routing/route_fuzz_targets.rs"]
mod route_fuzz_targets;

#[path = "routing/route_inspection_output.rs"]
mod route_inspection_output;

#[path = "routing/route_law_consistency.rs"]
mod route_law_consistency;

#[path = "routing/routing_fixture_sets.rs"]
mod routing_fixture_sets;

#[path = "routing/schema_snapshots.rs"]
mod schema_snapshots;

#[path = "routing/serde_roundtrip.rs"]
mod serde_roundtrip;

