---
title: Security Model
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Security Model

`bijux-dag` enforces declared policy gates, rooted path authorization, and
secret redaction for a local runtime. It does not claim full host sandboxing.

## Scope

This contract covers environment shaping, declared environment bindings,
policy-based effect denial, rooted input and output authorization, secret
redaction, and incident-oriented secret handling implemented in:

- `crates/bijux-dag-runtime/src/internal/identity/security_env.rs`
- `crates/bijux-dag-runtime/src/artifacts/storage/path_authorization.rs`

## Threat model

The governed threat surfaces are:

- undeclared ambient environment exposure to runtime tasks
- path traversal and symlink escape outside authorized input or output roots
- operator-facing leakage of obvious secret material in logs and payloads
- policy drift that claims hermetic behavior stronger than the runtime proves

This model does not claim containment against arbitrary host compromise,
kernel-level escape, unrestricted host filesystem reads, or real network and
clock virtualization.

## Hermeticity model

`--hermetic` in `bijux-dag` is a policy profile, not a full sandbox.

- declared effect policy can deny network, environment, and clock usage
- clean environment shaping can remove undeclared ambient keys
- rooted path authorization constrains governed input and output paths
- host process execution still occurs on the local machine

Non-hermetic behavior that remains intentionally outside the current proof
boundary is tracked in `docs/reports/governance/NON_HERMETIC_BEHAVIORS.md`.

## Environment controls

- `shape_environment` applies allowlist and denylist filtering before explicit
  bindings are materialized
- `declared_environment` exposes only declared keys when bindings are present
- `effective_env_allowlist` merges node-level and container-level environment
  declarations
- exact required bindings that are absent from ambient input must be reported

## Filesystem controls

- `authorize_input_path` must reject candidate paths that escape the canonical
  input root
- `authorize_output_path` must reject candidate paths that escape the canonical
  output root
- symlink-based output escape must be rejected after canonical resolution

## Secret handling and redaction

- operator-facing redaction must remove obvious secret payload values
- leak conformance checks must fail payloads that still contain common
  secret-bearing fields such as `token`, `password`, or `secret`
- secret scope, delivery mode, secure-mode selection, and incident response
  actions remain explicit typed surfaces enforced by runtime contract tests

## Related tests

- `crates/bijux-dag-runtime/tests/security_model_contracts.rs`
- `crates/bijux-dag-runtime/tests/security_policy_contracts.rs`
- `crates/bijux-dag-runtime/tests/secrets_security_contracts.rs`

## Versioning and change policy

Any incompatible change to hermeticity claims, environment shaping rules,
path-authorization behavior, or secret-redaction semantics must update this
contract and the linked runtime tests in the same change.
