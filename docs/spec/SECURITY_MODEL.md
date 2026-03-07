# Security Model

## Scope and authority
This document defines enforced security behavior for local execution in `bijux-dag`.
Future execution modes may add controls, but claims in this document are limited to
implemented runtime and policy surfaces.

## Threat model
The model assumes the following adversarial or failure scenarios:
- malicious DAG author attempting undeclared effects or path escape
- malicious or compromised adapter attempting secret leakage through logs/artifacts
- accidental leakage via ambient environment capture
- corrupted cache/run artifacts causing false reuse or unsafe replay
- untrusted input paths and symlink redirection

## Hermeticity model
Hermeticity is policy-driven and partial:
- `clean_env` can remove ambient environment inheritance
- `deny_env`, `deny_network`, and `deny_clock` can block declared effects
- output and storage path boundaries are enforced through path authorization and
  relative-path validation
- full host isolation is not guaranteed for all backends

See `docs/tracking/NON_HERMETIC_BEHAVIORS.md` for explicit known gaps.

## Environment controls
Environment shaping is centralized in runtime env policy helpers:
- allowlist patterns support exact keys and prefix (`NAME_*`) forms
- deny patterns override allowlist matches
- explicit per-node variables are filtered through same allow/deny rules
- `clean_env=true` drops ambient variables before allowlist filtering

## Filesystem controls
Path authorization is centralized and enforced for input/output roots:
- canonicalized candidate paths must remain inside authorized canonical root
- traversal and escape paths are rejected
- symlink escape paths are rejected because canonical targets are validated
- storage relative-path APIs reject absolute, traversal, and backslash forms

## Network controls
Network access is denied when policy sets `deny_network`. Behavior is enforced at
effect-policy layer for nodes declaring network effect. Backend-specific isolation
capabilities are documented separately and must not be implied here.

## Secret handling and redaction
Runtime security includes secret redaction and leakage checks:
- masking and secret leak checks are covered in `secrets_security_contracts`
- logs, diagnostics, and manifests must avoid raw secret values
- incident response surfaces require explicit containment actions

## Required security verification surfaces
- `crates/bijux-dag-runtime/tests/security_model_contracts.rs`
- `crates/bijux-dag-runtime/tests/secrets_security_contracts.rs`
- `crates/bijux-dev-dag` repo suite `security-model`

## Versioning and change policy
- Policy tightening is allowed in minor releases when behavior is documented.
- Policy loosening requires explicit contract update and linked tests.
- New security claims require corresponding control-plane evidence before merge.
