# App E2E Fast-Lane Completion Report

This report maps tasks 261-280 to repository artifacts.

## 261-266 lane review, promotion, and split

- lane classification:
  - `docs/reports/foundation/app_e2e_lane_classification.md`
- promoted fast-core smoke paths:
  - `crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs`
- suite split:
  - `configs/suites/app_e2e_fast_core.json`
  - `configs/suites/app_e2e_slow_extended.json`

## 267-270 generated scenario reports

- `docs/reports/foundation/app_fast_lane_skipped_scenarios_with_reasons.md`
- `docs/reports/foundation/app_promotable_skipped_scenarios.md`
- `docs/reports/foundation/app_slowest_full_lane_scenarios.md`
- `docs/reports/foundation/app_high_value_not_in_fast_lane.md`

## 271-272 budget and lane rationale governance

- `configs/policy/app_e2e_fast_lane_budget.json`
- `configs/policy/app_e2e_lane_rationale.json`
- `crates/bijux-dev-dag/tests/app_e2e_lane_governance_contracts.rs`

## 273-278 routed smoke coverage

- `crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs`
- `docs/reports/foundation/app_smoke_release_coverage_report.md`

## 279 canonical fixture

- `crates/bijux-dag-app/tests/fixtures/git_for_computation_graphs_workflow.json`

## 280 release gate for smoke domain span

- `crates/bijux-dev-dag/tests/app_e2e_lane_governance_contracts.rs`
- `docs/reports/foundation/app_smoke_release_coverage_report.md`
