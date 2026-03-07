# Execution semantics contract

## Scope
Defines execution semantics for planning, ordering, failure/retry behavior, selection, replay, and run integrity invariants.

## Planning and ordering
- Plan construction and execution ordering are independent contract surfaces.
- Deterministic workloads must produce equivalent outcomes for `jobs=1` and `jobs>1`.
- Equal-priority ready nodes use stable tie breaking.

## Failure and retry semantics
- Failure propagation is deterministic from state and dependency graph.
- Retry attempts and backoff metadata are persisted in trace artifacts.
- Timeout failures are explicitly distinguishable from execution failures.
- Cancellation must emit a complete final manifest.

## Selection and replay semantics
- Selection/exclusion still emits trace-complete manifests.
- Replay must not consult ambient state outside recorded artifacts and policy/config inputs.

## Integrity semantics
- Latest symlink updates must not mutate historical run directories.
- Run ID collision handling must be deterministic and safe.
- Manifest node totals must equal trace status totals.

## Related tests
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/fault_resilience_integration.rs`
- `evidence/battle/workflows/happy_path/*`

## Versioning and change policy
Execution semantic changes require matching updates to state-machine docs, runtime tests, and replay behavior tests in one change.
