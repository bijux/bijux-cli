# Foundation Backlog Routing Ledger

Source goals: repository foundation backlog baseline and owning contracts.
Issue class contract: `contracts/foundation/backlog_issue_class_routing.v1.json`.

| Goal | Issue Class | Owning Crate | Evidence Location | Status | Note |
| --- | --- | --- | --- | --- | --- |
| 1 | foundation-ownership-boundary | bijux-dev | contracts/foundation/workspace_product_map.v1.json | done | ownership contract and boundary test enforced |
| 2 | foundation-ownership-boundary | bijux-dev | crates/bijux-dev/tests/maintainer/architecture/ownership_boundaries.rs | done | maintainer import boundaries enforced |
| 3 | foundation-ownership-boundary | bijux-dev | contracts/foundation/dag_dependency_direction.v1.json | done | DAG dependency direction contract and tests enforced |
| 4 | foundation-ownership-boundary | bijux-dev | contracts/foundation/cli_dependency_direction.v1.json | done | CLI dependency direction contract and route boundary tests enforced |
| 5 | foundation-ownership-boundary | bijux-dev | contracts/foundation/module_surface_lanes.v1.json | done | module lane contract and export checks enforced |
| 7 | foundation-backlog-governance | bijux-dev | contracts/foundation/backlog_issue_class_routing.v1.json | done | routing contract and maintainer validation enforced |
| 8 | foundation-backlog-governance | bijux-dev | contracts/foundation/root_policy_surface_inventory.v1.json | done | root policy inventory contract and report mapping enforced |
| 9 | foundation-compatibility-lanes | bijux-cli | contracts/foundation/version_compatibility_lanes.v1.json | done | compatibility lanes, fixtures, and query contract enforced |
| 10 | foundation-release-gate | bijux-dev | crates/bijux-dev/tests/foundation_hard_release_gate_contracts.rs | done | deterministic hard gate workflow enforced through fixture gate |
| 28 | foundation-operator-diagnostics | bijux-cli | crates/bijux-cli/src/interface/cli/handlers/cli.rs | done | doctor findings now expose severity, surface, evidence path, remediation |
