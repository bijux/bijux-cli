# Environment Drift Benchmarks

Generated: 2026-03-08

## Scope
This report tracks the cost and signal quality of environment drift detection for:
- replay mismatch classification
- import/export portability verification
- hermetic env shaping on runtime boundaries

## Benchmark scenarios
- replay environment drift classification latency
- clean-env shaping throughput over large ambient env maps
- allowlist and denylist filtering overhead under wildcard-heavy policies
- imported-run verify-only preservation checks with environment markers

## Current status
- benchmark contract surfaces exist and are release-visible
- replay environment drift reporting is covered by direct tests
- no release-blocking regressions are recorded in this report snapshot

## Reference contract surfaces
- `crates/bijux-dag-app/src/replay/diff.rs`
- `crates/bijux-dag-runtime/src/internal/identity/security_env.rs`
- `crates/bijux-dag-runtime/tests/security_model_contracts.rs`
- `crates/bijux-dev-dag/tests/environment_identity_completion_contracts.rs`
