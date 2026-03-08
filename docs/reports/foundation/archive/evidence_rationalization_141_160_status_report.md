# Evidence Rationalization Status Report (141-160)

Generated: 2026-03-08

This report maps tasks 141-160 to shipped evidence classification policy, generated outputs,
governance contracts, and ADR coverage.

## 141-145 evidence command and report classification

- Policy source:
  - `configs/policy/evidence_rationalization_policy.json`
- Generated command lists:
  - `docs/reports/foundation/release_critical_evidence_commands_only_report.md`
  - `docs/reports/foundation/release_supporting_evidence_commands_report.md`
  - `docs/reports/foundation/advisory_only_evidence_commands_report.md`

## 146-148 duplicate-output rationalization

- `docs/reports/foundation/evidence_outputs_duplicate_signal_report.md`
- `docs/reports/foundation/evidence_report_consolidation.md`

## 149 machine-stability tests for release-critical outputs

- `crates/bijux-dev-dag/tests/evidence_rationalization_contracts.rs`
- `crates/bijux-dev-dag/tests/evidence_lane_classification_contracts.rs`

## 150 advisory isolation from blockers

- `crates/bijux-dev-dag/tests/evidence_lane_classification_contracts.rs`
- `docs/reports/foundation/release_gate_blocking_vs_advisory_report.md`

## 151-152 governance rules (severity/audience and docs mapping)

- `configs/policy/evidence_rationalization_policy.json`
- `docs/reports/foundation/evidence_docs_mapping_report.md`

## 153-154 docs-to-evidence and evidence-to-suite mappings

- `docs/reports/foundation/evidence_docs_mapping_report.md`
- `docs/reports/foundation/evidence_suite_exercise_mapping_report.md`

## 155 commands not exercised in CI

- `docs/reports/foundation/evidence_commands_not_exercised_in_ci_report.md`

## 156 release-critical gate coverage in green paths

- `crates/bijux-dev-dag/tests/evidence_rationalization_contracts.rs`
- `crates/bijux-dev-dag/tests/evidence_lane_classification_contracts.rs`

## 157-158 compact evidence packs

- `docs/reports/foundation/compact_release_evidence_pack.md`
- `docs/reports/foundation/compact_release_evidence_pack.json`
- `docs/reports/foundation/compact_advisory_evidence_pack.md`
- `docs/reports/foundation/compact_advisory_evidence_pack.json`

## 159 low-decision-value evidence output report

- `docs/reports/foundation/top_25_evidence_outputs_low_decision_value_report.md`

## 160 ADR

- `docs/adr/20260308-evidence-severity-rationalization.md`
