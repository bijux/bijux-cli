# Dev DAG Contraction Completion Report (361-380)

This report maps TODO 361-380 to existing code decomposition, direct tests, reporting, and governance.

## 361-374 command split and direct coverage

- command decomposition and routing support:
  - `crates/bijux-dev-dag/src/commands/perf_evidence.rs`
  - `crates/bijux-dev-dag/src/commands/suite_catalog.rs`
  - `crates/bijux-dev-dag/src/commands/reporting.rs`
  - `crates/bijux-dev-dag/src/commands/command_runtime.rs`
  - `crates/bijux-dev-dag/src/commands/shared_io.rs`
  - `crates/bijux-dev-dag/src/commands/evidence_registry.rs`
- repo/report/tooling helpers:
  - `crates/bijux-dev-dag/src/repo/layout.rs`
  - `crates/bijux-dev-dag/src/repo/root.rs`
  - `crates/bijux-dev-dag/src/report/write.rs`
  - `crates/bijux-dev-dag/src/tooling/cargo.rs`
  - `crates/bijux-dev-dag/src/tooling/git.rs`
  - `crates/bijux-dev-dag/src/tooling/mod.rs`
- coverage and safety gates:
  - `crates/bijux-dev-dag/tests/dev_dag_direct_test_presence_contracts.rs`
  - `crates/bijux-dev-dag/tests/dev_dag_command_safety_contracts.rs`
  - `crates/bijux-dev-dag/tests/file_size_guardrails.rs`

## 375-378 reporting surfaces

- dev-dag hot files report:
  - `docs/reports/foundation/dev_dag_hot_files_report.md`
- dev-dag low coverage files report:
  - `docs/reports/foundation/dev_dag_low_coverage_files_report.md`
- release-critical evidence commands only report:
  - `docs/reports/foundation/release_critical_evidence_commands_only_report.md`
- advisory-only evidence commands report:
  - `docs/reports/foundation/advisory_only_evidence_commands_report.md`

## 379 release gate

- release-critical evidence execution gate:
  - `crates/bijux-dev-dag/tests/evidence_lane_classification_contracts.rs`

## 380 ADR

- `docs/adr/20260308-dev-dag-cleanup-end-state.md`
