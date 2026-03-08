# Runtime Scope Contraction Status Report (401-420)

## 401-404 inventory and classification

- Runtime module inventory source: `configs/policy/runtime_scope_v2.json`
- Lifecycle classification policy: `configs/policy/runtime_module_lifecycle_status.json`
- Scope classification report: `docs/reports/foundation/runtime_scope_classification_report.md`
- Ownership and quarantine references:
  - `configs/policy/runtime_broad_surface_ownership.json`
  - `docs/reports/foundation/runtime_quarantined_owner_repo_map.md`

## 405-413 quarantine and non-leakage controls

- Quarantined namespace prefixes governed in lifecycle policy.
- Stable surface and default UX non-leakage enforcement:
  - `crates/bijux-dev-dag/tests/runtime_scope_contraction_401_420_contracts.rs`
  - `crates/bijux-dev-dag/tests/runtime_scope_contraction_101_120_contracts.rs`
  - `crates/bijux-dev-dag/tests/runtime_overreach_contracts.rs`

## 414-416 generated surface reports

- Stable vs experimental page: `docs/reports/foundation/runtime_stable_vs_experimental_surface_page.md`
- Runtime public-surface size report: `docs/reports/foundation/runtime_public_surface_size_report.md`
- Runtime public-surface shrink trend report: `docs/reports/foundation/runtime_public_surface_shrink_trend_report.md`

## 417-420 governance and closure

- Governance rules: `docs/spec/RUNTIME_SCOPE_GOVERNANCE_POLICY.md`
- Verification suite: `configs/suites/runtime_scope_contraction_verification.json`
- Architecture ADR: `docs/adr/20260308-runtime-scope-end-state.md`
