# Invariant Coverage Tracking

## Coverage summary

| ID | Coverage status | Source |
| --- | --- | --- |
| INV-GRAPH-SHAPE-001 | enforced | graph validation + `formal_invariant_property_contracts.rs` |
| INV-GRAPH-SHAPE-002 | enforced | graph validation |
| INV-GRAPH-SHAPE-003 | enforced | `formal_invariant_property_contracts.rs` |
| INV-PLAN-SHAPE-001 | enforced | planner determinism tests |
| INV-SCHED-READY-001 | enforced | scheduler contract tests |
| INV-SCHED-STATE-001 | enforced | scheduler contract tests |
| INV-RUN-COUNTS-001 | enforced | runtime invariant check in `verify_run` |
| INV-RUN-TERMINAL-001 | enforced | runtime invariant check in `verify_run` |
| INV-TRACE-TIME-001 | enforced | runtime invariant check in `verify_run --deep` |
| INV-TRACE-ATTEMPT-001 | partial | trace schema checks present; stronger causal checks pending |
| INV-CACHE-PROOF-001 | enforced | cache evolution contract tests |
| INV-ARTIFACT-REF-001 | enforced | `verify_run` output file checks |
| INV-ARTIFACT-PATH-001 | enforced | `verify_run --deep` path normalization checks |
| INV-EXPORT-VERIFY-001 | partial | import/export coverage present; consolidated invariant report pending |
| INV-REPLAY-EQUIV-001 | enforced | run diff replay equivalence checks |

## Missing enforcement focus
- INV-TRACE-ATTEMPT-001: add explicit cross-attempt ordering checks.
- INV-EXPORT-VERIFY-001: unify import/export invariant checks into shared invariant entrypoint.
