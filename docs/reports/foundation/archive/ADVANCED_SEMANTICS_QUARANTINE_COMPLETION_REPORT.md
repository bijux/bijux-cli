# Advanced Semantics Quarantine Completion Report (341-360)

This report maps TODO 341-360 to existing inventory, quarantine, invariants, and governance artifacts.

## 341-346 inventory and classification

- inventory by internal family:
  - `docs/reports/foundation/RUNTIME_INTERNAL_SURFACE_INVENTORY_REPORT.md`
- broad inventory and ownership surfaces:
  - `docs/reports/foundation/ADVANCED_SEMANTICS_INVENTORY.md`
  - `docs/reports/foundation/RUNTIME_BROAD_SURFACE_INVENTORY.md`
- classification categories enforced in policy and contracts:
  - `configs/policy/advanced_semantics_governance.json`
  - `crates/bijux-dev-dag/tests/advanced_semantics_governance_contracts.rs`

## 347-349 missing tests/paths/fixtures reports

- no tests report: `docs/reports/foundation/advanced_semantics_no_direct_tests_report.md`
- no user path report: `docs/reports/foundation/advanced_semantics_no_user_path_report.md`
- no fixtures report: `docs/reports/foundation/advanced_semantics_no_examples_report.md`

## 350-356 quarantine and invariant guarantees

- quarantine review: `docs/reports/foundation/advanced_semantics_quarantine_review.md`
- invariants:
  - advanced semantics do not affect graph identity unless intended
  - advanced semantics do not affect replay proof unless intended
  - advanced semantics do not appear in default help/capability output
- retained fixtures by family:
  - `docs/reports/foundation/advanced_semantics_retained_examples.md`

## 357-359 budget/governance/ADR

- budget report: `docs/reports/foundation/speculative_surface_budget.md`
- governance gate: `crates/bijux-dev-dag/tests/advanced_semantics_progress_contracts.rs`
- ADR: `docs/adr/ADR-ADVANCED-SEMANTICS-END-STATE.md`

## 360 stable vs experimental page

- `docs/reports/foundation/RUNTIME_STABLE_VS_EXPERIMENTAL_SURFACE_PAGE.md`
