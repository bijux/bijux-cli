# Config Mutation Hardening Law

Config mutation commands (`set`, `clear`, `unset`) are required to remain safe under corruption, write failure, and retries.

Frozen requirements:

1. Corrupted config input must produce deterministic diagnostics.
2. Failed mutation attempts must preserve last known-good file content.
3. Retry after transient write failure must be idempotent.
4. Concurrent reads and writes must not produce malformed config shape.
5. State diagnostics must surface config corruption evidence.

Evidence sources:

- `artifacts/status/config_corruption_matrix.json`
- `artifacts/status/config_rollback_proof.json`
- `crates/bijux-cli/tests/integration/cli/resilience/config_corruption_hardening.rs`
