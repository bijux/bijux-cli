# Repo Tree Governance Status Report (181-200)

Generated: 2026-03-08

This report maps tasks 181-200 to shipped module inventory artifacts, governance
policy, enforcement contracts, dashboard outputs, and architecture decisions.

## 181-184 module size and direct-test inventory reports

- `docs/reports/foundation/MODULE_INVENTORY_UNDER_10_LINES_REVIEW.md`
- `docs/reports/foundation/MODULE_INVENTORY_OVER_500_LINES.md`
- `docs/reports/foundation/module_inventory_over_1000_lines.md`
- `docs/reports/foundation/MODULE_INVENTORY_ZERO_DIRECT_TESTS.md`

## 185-187 coverage, fixture, and docs-link reports

- `docs/reports/foundation/module_low_coverage_high_churn_report.md`
- `docs/reports/foundation/module_no_linked_fixtures_report.md`
- `docs/reports/foundation/module_no_linked_docs_report.md`

## 188-190 governance rules for ownership and module sizing

- `configs/policy/module_hygiene_governance.json`
- Governance rules include:
  - top-level module ownership classification requirement
  - large-module split-rationale requirement
  - tiny-wrapper justification requirement

## 191-193 naming and unused-surface reports

- `docs/reports/foundation/module_name_oversell_report.md`
- `docs/reports/foundation/module_rename_alignment_report.md`
- `docs/reports/foundation/dead_reexports_unused_preludes_report.md`

## 194-195 duplicate helper inventories

- `docs/reports/foundation/duplicate_helper_modules_report.md`
- `docs/reports/foundation/TOP_25_DUPLICATE_HELPER_AREAS_REPORT.md`

## 196-197 repo-tree hotspot and cleanup-candidate pages

- `docs/reports/foundation/repo_tree_hotspots_report.md`
- `docs/reports/foundation/repo_tree_cleanup_candidates_report.md`

## 198 module-hygiene drift gate

- `docs/reports/foundation/module_hygiene_drift_gate_report.md`
- `crates/bijux-dev-dag/tests/module_hygiene_governance_contracts.rs`

## 199 maintainer repo-tree health dashboard

- `docs/reports/foundation/repo_tree_health_dashboard.md`
- `crates/bijux-dev-dag/tests/evidence_dashboard_contracts.rs`

## 200 ADR for target repo-tree shape

- `docs/adr/20260308-REPO-TREE-SHAPE-TARGET-V0-1-0.md`
