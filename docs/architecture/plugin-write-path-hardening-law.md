# Plugin Write-Path Hardening Law

Plugin lifecycle mutations (`install`, `uninstall`, `enable`, `disable`) are release-blocking surfaces.

The hardening law is fixed:

1. Every registry mutation must be atomic and rollback-safe.
2. A failed mutation must preserve the last healthy registry state.
3. Retry after transient write failure must be idempotent.
4. `plugins check` must reject post-install manifest drift and unsupported runtime kinds.

Evidence sources:

- `artifacts/status/plugin_lifecycle_failure_injection_report.json`
- `artifacts/status/plugin_rollback_proof_report.json`
- `crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs`
- `crates/bijux-cli-bin/tests/plugin_failure_injection.rs`
