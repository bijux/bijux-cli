# Secure DAG authoring patterns

This guide defines secure DAG authoring patterns and anti-patterns for secret-bearing workflows.

## Recommended patterns

- use `SecretReference` contracts instead of plain params/env literals
- resolve secrets at runtime unless compile-time resolution is explicitly allowed
- prefer file-mount or backend-native secret delivery in hardened environments
- enable full masking for logs, diagnostics, manifests, and export bundles
- pin secret versions for backfills and long-lived replay-critical workflows

## Anti-patterns

- passing secrets as process arguments
- writing secrets to stdout/stderr
- embedding secret values in artifact manifests
- storing unclassified secret-derived artifacts in general artifact stores
- using local-auth bypass semantics in non-local environments

## Secret-taint governance

Secret-bearing nodes should mark taint for logs, diagnostics, and artifacts.
Tainted outputs must inherit stricter retention/export controls.

## Secure execution mode

Production environments should enable strict secure mode with hardened policy bundles and teardown guarantees.

## Conformance fixtures

- `crates/bijux-dag-runtime/tests/fixtures/secrets/stdout_leak.json`
- `crates/bijux-dag-runtime/tests/fixtures/secrets/stderr_leak.json`
- `crates/bijux-dag-runtime/tests/fixtures/secrets/env_dump_leak.json`
- `crates/bijux-dag-runtime/tests/fixtures/secrets/panic_payload_leak.json`
- `crates/bijux-dag-runtime/tests/fixtures/secrets/artifact_manifest_leak.json`
