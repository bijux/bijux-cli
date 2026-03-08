# Environment Identity Contract

## Scope
Environment identity defines the deterministic execution-context identity used for
run-level provenance, replay analysis, and cache safety decisions.

This contract is limited to currently implemented behavior in:
- `crates/bijux-dag-runtime/src/internal/identity/security_env.rs`
- runtime/app replay and import/export contract tests

## Canonical inputs
Environment identity is composed from normalized execution context fields:
- shaped environment key/value map after `clean_env`, allowlist, and denylist filters
- declared backend identity (local, container, remote, kubernetes, hpc) where included
- toolchain and runtime version markers when supplied by runtime metadata
- explicit run-level environment policy controls

## Determinism rules
- Environment key ordering must not affect identity.
- Equivalent allowlist/denylist results must produce equivalent identity.
- Denylist filtering has precedence over allowlist admission.
- Explicit env values override ambient values for the same admitted key.
- Identity changes when admitted variable values change.
- Identity changes when declared toolchain markers change.
- Identity may change when backend identity is intentionally modeled as part of run identity.

## Hermeticity and leakage guarantees
- `clean_env=true` removes ambient environment inheritance.
- Ambient variables not admitted by allowlist are excluded.
- Denied variables are excluded even when allowlisted.
- Replay and imported-run flows must not recover omitted ambient variables.

## Explainability requirement
Operator explain surfaces must expose environment drift as a first-class reason for
replay mismatch and cache miss diagnostics.

## Required verification surfaces
- `crates/bijux-dag-runtime/tests/security_model_contracts.rs`
- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dev-dag/tests/environment_identity_completion_contracts.rs`

## Stability level
Stable for `v0.1` operator and governance surfaces.
