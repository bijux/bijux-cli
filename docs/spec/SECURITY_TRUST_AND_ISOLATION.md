# SECURITY TRUST AND ISOLATION

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/BATTLE_TRUST_PROPERTIES.md
# Battle trust properties

## Scope

This document defines the canonical trust properties for battle workflow evidence.

## Canonical trust properties

- `tp_deterministic_scheduling`
- `tp_failure_propagation`
- `tp_replay_equivalence`
- `tp_cache_integrity`
- `tp_artifact_integrity`
- `tp_policy_enforcement`
- `tp_operator_observability`
- `tp_import_export_compatibility`
- `tp_state_machine_legality`
- `tp_timeout_retry_accounting`
- `tp_secret_redaction`
- `tp_run_dir_resilience`
- `tp_plan_truth`

## Authority and mapping

The normative source for trust property metadata and scenario mapping is [`configs/policy/battle_trust_properties.json`](../../configs/policy/battle_trust_properties.json).

## Governance rules

- Every battle workflow scenario must map to one or more canonical trust properties.
- No battle scenario is admitted without an owner and a `why_exists` statement.
- Drift checks must reject orphan scenarios and unknown trust property identifiers.
- Foundation suite execution includes battle trust coverage checks.

## SOURCE: docs/spec/DNA_EXECUTION_CONTRACT.md
# DNA Execution Contract

## Purpose
Define allowed `bijux-dna` execution extensions and strict boundaries.

## DNA may extend
- HPC/scheduler integration details
- queue/account/partition metadata
- scheduler-native evidence fields

## DNA may not redefine
- shared identity contracts
- replay fidelity levels
- artifact lineage semantics

## SOURCE: docs/spec/ENVIRONMENT_IDENTITY_CONTRACT.md
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

## SOURCE: docs/spec/SANDBOX_SECURITY_MODEL_CONTRACT.md
# Sandbox Security Model Contract

## Purpose

Define enforced sandbox and execution-boundary security guarantees.

## Required isolation surfaces

- container backend isolation
- shell backend isolation
- remote backend isolation
- environment leakage prevention
- filesystem boundary enforcement

## Required adversarial protections

- symlink escape prevention
- path traversal prevention
- command injection prevention
- runtime argument sanitization
- artifact read/write boundary enforcement

## Required policy controls

- backend privilege restriction enforcement
- sandbox policy enforcement diagnostics
- sandbox failure detection behavior
- adversarial execution verification coverage

## Governance artifacts

- sandbox security regression corpus
- sandbox hardening stress suite
- sandbox benchmark report
- sandbox telemetry report

## SOURCE: docs/spec/SECURITY_MODEL.md
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

## SOURCE: docs/spec/TEST_TRUST_CONTRACT.md
# Test trust contract

## Scope

This contract defines minimum runtime trust evidence requirements.

## Required evidence classes

- semantic
- adversarial
- failure
- replay mismatch
- scheduler edge behavior
- policy violation
- cache poisoning defense
- artifact corruption handling
- cancellation terminal behavior
- state machine consistency
- recovery behavior
- import/export manifest checks
- node execution behavior
- scheduler determinism

## Catalog source

Required suites are listed in `crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json`.

## Enforcement

Control-plane suite `test-trust-foundation` verifies contract docs and catalog-backed files exist.

## SOURCE: docs/spec/TEST_TRUST_LEDGER.md
# Test trust ledger

## Scope

This ledger defines trust-value classification and mandatory semantic surfaces for runtime tests.

## Trust-value classes

- `critical`: must-never-break trust proofs tied to correctness and safety boundaries.
- `useful`: meaningful contract coverage that supports regression detection.
- `shallow`: low-depth checks that remain only when they guard discoverability or catalog integrity.
- `cosmetic`: non-semantic checks; forbidden as progress metrics.
- `duplicate`: overlapping checks superseded by stronger trust surfaces.

## Normative policy

The normative policy file is `configs/policy/test_trust_ledger.json`.

## Enforcement rules

- Every runtime test file must be classified by exactly one trust-value class.
- `must_never_break` entries must exist and cannot be classified as `cosmetic` or `duplicate`.
- Required semantic surfaces must exist and remain mapped.
- Forbidden snapshot macros are rejected outside explicit allowlisted files.
- Foundation and repo governance checks must include the test-trust cleanup guard.
