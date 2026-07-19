# Application Contract Tests

These integration tests verify the boundary where DAG behavior becomes an
operator-facing command result. They exercise `bijux-dag-app` directly; binary
packaging belongs to `bijux-dag-cli`, while scheduling, storage, and graph
semantics remain owned by their lower-level crates.

## Coverage

- command routing, argument resolution, and stable output envelopes
- graph validation, planning, execution, replay, and repair orchestration
- retained runs, artifact inspection, lineage, comparison, and promotion
- configuration precedence, policy exposure, and backend selection
- repository-backed branch, cache, container, data, and failure workflows
- malformed-input and no-panic behavior at operator entrypoints

Tests ending in `_contract.rs` protect a named public or cross-crate contract.
Workflow tests should use fixtures from `evidence/dag/` rather than reconstruct
a weaker private copy. Snapshot changes require an explanation of the
operator-visible change; regenerating output is not sufficient justification.

## Focused Runs

```bash
cargo nextest run -p bijux-dag-app --test output_contract
cargo nextest run -p bijux-dag-app --test cache_behavior_workflow_contract
cargo nextest run -p bijux-dag-app --test replay_contract
```

Use the exact failing test binary first. Run the package only after the focused
contract is green. Repository-wide policy checks for these surfaces live in
`crates/bijux-dev/tests/`.

## Failure Interpretation

A failure usually indicates one of three defects: orchestration no longer
matches the underlying crate contract, a stable response changed without its
schema and documentation, or governed evidence drifted from the implementation.
Fix the owning boundary. Do not weaken assertions or replace deterministic
fixtures with timing retries.
