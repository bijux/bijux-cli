# Security Debt Ledger

This ledger tracks unresolved security work with explicit ownership.

| id | debt item | owner surface | severity | blocker |
| --- | --- | --- | --- | --- |
| SEC-001 | Enforce deny-network with backend-level isolation for every backend type | `crates/bijux-dag-runtime/src/execution_backend.rs` | high | release blocker |
| SEC-002 | Add end-to-end undeclared output rejection tests across all execution backends | `tests/e2e/policy/` | medium | release blocker |
| SEC-003 | Add policy-audit parity checks between CLI/app/runtime resolved policy state | `crates/bijux-dag-app` + `crates/bijux-dev-dag` | medium | non-blocking |
| SEC-004 | Expand symlink/path authorization checks into import/export bundle paths | `crates/bijux-dag-runtime/src/store.rs` | high | release blocker |
| SEC-005 | Add automated redaction snapshot suite for logs/traces/errors in failing runs | `crates/bijux-dag-runtime/tests` | high | release blocker |
