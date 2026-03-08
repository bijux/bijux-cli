# Runtime Stable vs Experimental Surface Page

Generated from code annotations and governance policy:

- `configs/policy/runtime_scope_v2.json`
- `configs/policy/advanced_semantics_governance.json`

## Stable surfaces

Stable surfaces are represented by retained categories and code-backed paths with user-facing routes:

- `kernel-relevant`
- `runtime-relevant`
- `adapter-relevant`

Reference detail reports:

- `docs/reports/foundation/runtime_stable_surfaces.md`
- `docs/reports/foundation/runtime_operator_surface_report.md`

## Experimental surfaces

Experimental surfaces are represented by governed speculative entries:

- `speculative`

Constraints:

- explicit owner and lifecycle (`expire-or-graduate`)
- target date required
- no default user-facing path

Reference detail reports:

- `docs/reports/foundation/runtime_experimental_surfaces.md`
- `docs/reports/foundation/speculative_surface_budget.md`
- `docs/reports/foundation/advanced_semantics_no_user_path_report.md`

## Governance gates

- `crates/bijux-dev-dag/tests/advanced_semantics_governance_contracts.rs`
- `crates/bijux-dev-dag/tests/advanced_semantics_progress_contracts.rs`
- `crates/bijux-dev-dag/tests/advanced_semantics_end_state_contracts.rs`
