# Formal Invariants

## Invariant registry

| ID | Domain | Invariant | Enforcement |
| --- | --- | --- | --- |
| INV-GRAPH-SHAPE-001 | graph | graph is acyclic and references are valid | graph validation + generated-shape tests |
| INV-GRAPH-SHAPE-002 | graph | node ids are unique | graph validation |
| INV-GRAPH-SHAPE-003 | graph | canonical order is stable | formal invariant property tests |
| INV-PLAN-SHAPE-001 | plan | executable plan contains deterministic dependency structure | planner tests + property tests |
| INV-SCHED-READY-001 | scheduler | downstream node becomes ready exactly once | scheduler contract tests |
| INV-SCHED-STATE-001 | scheduler | terminal node state does not revert | scheduler contract tests |
| INV-RUN-COUNTS-001 | run_state | manifest totals match node trace terminal statuses | runtime invariant check + `dag runs verify` |
| INV-RUN-TERMINAL-001 | run_state | completed run includes at least one terminal node status | `dag runs verify` |
| INV-TRACE-TIME-001 | trace | trace finished time is not before start time | `dag runs verify --deep` |
| INV-TRACE-ATTEMPT-001 | trace | trace attempt metadata is coherent for a node | trace schema + trace tests |
| INV-CACHE-PROOF-001 | cache | cache hit requires compatible proof metadata | cache evolution tests |
| INV-ARTIFACT-REF-001 | artifacts | indexed artifact entries point to existing files | `dag runs verify` |
| INV-ARTIFACT-PATH-001 | artifacts | indexed artifact paths are normalized relative paths | `dag runs verify --deep` |
| INV-EXPORT-VERIFY-001 | import_export | imported and exported bundles pass invariant checks before use | import/export contract tests |
| INV-REPLAY-EQUIV-001 | replay | replay comparison equivalence reasons are explicit and deterministic | replay diff tests |

## Coverage rule
- Every invariant ID must map to at least one code check, test, or control-plane guard.
- New normative guarantees must reference invariant IDs directly.

## Enforcement command
- `bijux-dev-dag invariants-report` prints the current registry and mapped coverage.

## Guarantee wording rule
Claims using words such as “guarantee”, “always”, or “never” in normative docs must cite at least one invariant ID in `INV-*` format.
