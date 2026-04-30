# Root Policy Surface Report

Source contract: `contracts/foundation/root_policy_surface_inventory.v1.json`.

This report tracks root policy files that gate product behavior and maps each to crate-owned enforcement surfaces.

| Policy File | Owning Crate | Gated Behavior | Enforcement Surface |
| --- | --- | --- | --- |
| contracts/foundation/backlog_issue_class_routing.v1.json | bijux-dev | Enforces backlog issue-class ownership and rejects uncategorized backlog intake. | crates/bijux-dev/tests/foundation_backlog_issue_routing_contracts.rs |
| contracts/foundation/cli_dependency_direction.v1.json | bijux-cli | Locks CLI dependency direction and DAG boundary isolation. | crates/bijux-cli/tests/architecture/boundaries/cli_dependency_direction_boundaries.rs |
| contracts/foundation/dag_dependency_direction.v1.json | bijux-dev | Locks DAG dependency direction and source import boundaries. | crates/bijux-dev/tests/foundation_dag_dependency_direction_contracts.rs |
| contracts/foundation/module_surface_lanes.v1.json | bijux-dev | Classifies stable, experimental, simulated, and private module lanes. | crates/bijux-dev/tests/foundation_module_surface_contracts.rs |
| contracts/foundation/root_policy_surface_inventory.v1.json | bijux-dev | Defines the root policy inventory boundary and ownership mapping itself. | crates/bijux-dev/tests/foundation_root_policy_surface_inventory_contracts.rs |
| contracts/foundation/workspace_product_map.v1.json | bijux-dev | Declares product ownership and maintainer boundaries. | crates/bijux-dev/tests/foundation_product_map_contracts.rs |
| contracts/foundation/version_compatibility_lanes.v1.json | bijux-cli | Codifies current, previous, and refused version lanes for core product surfaces. | crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs |
| contracts/official_product_namespace_registry.json | bijux-cli | Controls official namespace mounting and plugin collision policy. | crates/bijux-cli/tests/routing/laws/route_law_consistency.rs |
| contracts/product_mount_metadata_contract.json | bijux-cli | Defines required mount descriptor metadata for product routing. | crates/bijux-cli/src/contracts/product_mount.rs |
| contracts/schemas/error-envelope-v1.schema.json | bijux-cli | Fixes machine-readable CLI failure envelope shape. | crates/bijux-cli/tests/routing/registry/query_interfaces.rs |
| contracts/schemas/output-envelope-v1.schema.json | bijux-cli | Fixes machine-readable CLI success envelope shape. | crates/bijux-cli/tests/routing/registry/query_interfaces.rs |
| contracts/schemas/plugin-manifest-v2.schema.json | bijux-cli | Fixes plugin manifest compatibility and capability schema. | crates/bijux-cli/tests/integration/cli/plugins/plugin_cli_lifecycle.rs |
| configs/status/compatibility_baseline.json | bijux-dev | Pins compatibility warning baselines for quality suites. | crates/bijux-dev/src/maintainer/suites/quality/plugin_quality_compatibility.rs |
| configs/status/route_special_cases_baseline.json | bijux-dev | Pins route special-case baselines for command-surface quality. | crates/bijux-dev/src/maintainer/suites/quality/command_surface.rs |
