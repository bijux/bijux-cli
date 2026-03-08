# Dev-Dag Contraction Status Report (121-140)

Generated: 2026-03-08

This report maps tasks 121-140 to shipped command decomposition, direct helper coverage gates,
reporting artifacts, fast suite governance, and architecture ADRs.

## 121 decomposition of command surfaces

- Primary decomposition targets are present under:
  - `crates/bijux-dev-dag/src/commands/perf_evidence.rs`
  - `crates/bijux-dev-dag/src/commands/suite_catalog.rs`
  - `crates/bijux-dev-dag/src/commands/evidence_registry.rs`
  - `crates/bijux-dev-dag/src/commands/reporting.rs`
  - `crates/bijux-dev-dag/src/commands/command_runtime.rs`
  - `crates/bijux-dev-dag/src/commands/shared_io.rs`

## 122-133 direct test anchors for commands/repo/report/tooling

- Presence and direct-test gates:
  - `crates/bijux-dev-dag/tests/dev_dag_direct_test_presence_contracts.rs`
  - `crates/bijux-dev-dag/tests/dev_dag_helper_small_module_test_gate_contracts.rs`
  - `crates/bijux-dev-dag/tests/dev_dag_command_coverage_progress_contracts.rs`

## 134 dev-dag 0%-coverage report grouped by command family

- `docs/reports/foundation/dev_dag_zero_coverage_report_by_command_family.md`

## 135 dev-dag command-size report grouped by command family

- `docs/reports/foundation/dev_dag_command_size_report_by_family.md`

## 136 duplicate evidence/report output consolidation

- Consolidation evidence:
  - `docs/reports/foundation/dev_dag_command_module_boundaries.md`
  - `docs/reports/foundation/dev_dag_contraction_completion_report.md`

## 137-138 release and advisory signal dashboards

- Release-facing dashboard:
  - `docs/reports/foundation/evidence_dashboard.md`
- Advisory-facing dashboard:
  - `docs/reports/foundation/advisory_evidence_dashboard.md`

## 139 fast helper suite for repo/tooling/report modules

- `configs/suites/dev_dag_helpers_fast.json`
- `crates/bijux-dev-dag/tests/dev_dag_helpers_fast_suite_contracts.rs`

## 140 end-state ADR for command decomposition

- `docs/adr/20260308-dev-dag-command-decomposition-shape.md`
