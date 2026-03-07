# Artifact retention and cleanup policy

## Scope

This policy governs run artifacts, cache entries, promoted artifacts, and exported bundles.

## Retention classes

- `local cache`: short-lived, evictable, rebuildable.
- `run artifacts`: medium-lived evidence for replay and diagnostics.
- `promoted artifacts`: long-lived evidence and release assets.
- `exported bundles`: transport artifacts with bounded retention.

## Baseline defaults

Default retention values are defined by `RetentionPolicy` in
`crates/bijux-dag-artifacts/src/retention.rs`:

- local cache: 7 days
- run artifacts: 30 days
- promoted artifacts: 365 days
- exported bundles: 180 days

## Cleanup rules

- Cleanup must never delete active staging directories.
- Cleanup must never delete promoted artifacts before promoted retention expires.
- Cleanup must preserve run manifests and trace files for runs still within run-artifacts retention.
- Cleanup must be deterministic for equal input state and cutoff timestamps.

## Deferred implementation boundary

Garbage collection orchestration is documented and policy-backed, but full automated GC
execution can be deferred. Any future GC implementation must enforce this policy and emit
audit evidence for each deletion decision.
