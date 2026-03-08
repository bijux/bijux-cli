# Runtime Scope Contraction Status Report (101-120)

Generated: 2026-03-08

This report maps tasks 101-120 to shipped runtime-scope inventory, classification,
quarantine governance, and documentation artifacts.

## 101-106 inventory and classification

- Internal families are inventoried and classified in:
  - `configs/policy/runtime_scope_v2.json`
  - `docs/architecture/runtime_scope_v2.md`
  - `docs/reports/foundation/runtime_internal_surface_inventory_report.md`
- Coverage includes:
  - `internal/analysis`
  - `internal/control`
  - `internal/ext`
  - `internal/identity`
  - `internal/workflow`

## 107-109 no-tests / no-fixtures / no-user-path reports

- `docs/reports/foundation/advanced_semantics_no_direct_tests_report.md`
- `docs/reports/foundation/advanced_semantics_no_examples_report.md`
- `docs/reports/foundation/advanced_semantics_no_user_path_report.md`

## 110 quarantine movement and boundary enforcement

- Quarantine and ownership policy:
  - `configs/policy/advanced_semantics_governance.json`
  - `configs/policy/runtime_broad_surface_ownership.json`
- Quarantine review:
  - `docs/reports/foundation/advanced_semantics_quarantine_review.md`
  - `docs/reports/foundation/runtime_keep_quarantine_delete_review.md`

## 111-114 invariants for identity/replay/help/capabilities

- Governance contracts:
  - `crates/bijux-dev-dag/tests/advanced_semantics_governance_contracts.rs`
  - `crates/bijux-dev-dag/tests/advanced_semantics_end_state_contracts.rs`
- These contracts enforce that speculative/advanced surfaces do not leak into
  graph identity, replay proof, default CLI help, and default capability output.

## 115 retained examples by advanced family

- Retained fixtures:
  - `crates/bijux-dag-runtime/tests/fixtures/advanced_semantics/kernel_relevant_example.json`
  - `crates/bijux-dag-runtime/tests/fixtures/advanced_semantics/runtime_relevant_example.json`
  - `crates/bijux-dag-runtime/tests/fixtures/advanced_semantics/adapter_relevant_example.json`
- Index report:
  - `docs/reports/foundation/advanced_semantics_retained_examples.md`

## 116 quarantine/delete handling for unsupported surfaces

- Decision ledger and named decisions:
  - `configs/policy/runtime_scope_v2.json`
  - `docs/reports/foundation/advanced_semantics_quarantine_completion_report.md`

## 117 speculative-surface budget

- `docs/reports/foundation/speculative_surface_budget.md`

## 118 expire-or-graduate governance gate

- `crates/bijux-dev-dag/tests/advanced_semantics_progress_contracts.rs`

## 119 stable-vs-experimental surface page

- `docs/reports/foundation/runtime_stable_vs_experimental_surface_page.md`

## 120 end-state ADR

- `docs/adr/20260308-advanced-semantics-runtime-boundary.md`
- `docs/adr/ADR-advanced-semantics-end-state.md`
