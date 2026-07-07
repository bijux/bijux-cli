# Runtime Semantics Contract

## Scope

This contract defines the deterministic semantic helpers that the runtime uses
for scheduling order, retry behavior, cache validation, manifest verification,
recovery decisions, replay equivalence, and audit event accounting.

## Authoritative source

`crates/bijux-dag-runtime/src/runtime_core/governance/semantics.rs` is the
authoritative source for these semantic helper functions.

## Stable semantic surfaces

- deterministic schedule ordering
- retry and timeout decisions
- dependency-complete checks
- cache validity and invalidation
- artifact and manifest integrity checks
- replay equivalence decisions
- audit event categorization and counting

## Retry decision contract

Retry eligibility is a stable runtime semantic surface rather than an adapter
detail.

- policy failures are never retryable, even if a node declares policy failure
  classes in its retry params
- timeout retries are controlled by `timeout_retry_policy`, which supports
  `by_failure_class`, `always`, and `never`
- exit-code-specific retries are controlled by `retryable_exit_codes`
- retry reason is persisted in attempt evidence and retry lifecycle events so a
  later inspection can explain why the runtime scheduled or vetoed another
  attempt

## Related tests

- `crates/bijux-dag-runtime/tests/runtime_semantics_contracts.rs`
- `crates/bijux-dag-runtime/tests/engine_correctness_contracts.rs`

## Versioning and change policy

Runtime semantic helper names and their operator-visible decision meaning are
stable contract surfaces. Any incompatible change requires updating this
document and the linked runtime tests in the same change.
