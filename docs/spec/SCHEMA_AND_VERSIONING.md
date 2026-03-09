# SCHEMA AND VERSIONING

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ARTIFACT_INSPECT_SCHEMA_V0.1.md
# Artifact Inspect Schema v0.1

Command:

- `dag artifact-inspect <run_dir> <artifact_id> --json`

Schema:

- `configs/schema/operator/artifact_inspect.schema.json`

Required output fields:

- `artifact_id`
- `artifact_sha256`
- `node_id`
- `node_fingerprint`
- `path`
- `size_bytes`
- `provenance.graph_fingerprint`
- `provenance.run_id`
- `provenance.attempt`
- `lineage.upstream_artifact_ids`
- `lineage.downstream_artifact_ids`
- `identity_explain.artifact_id`
- `identity_explain.composed_from.run_id`
- `identity_explain.composed_from.node_id`
- `identity_explain.composed_from.node_fingerprint`
- `identity_explain.composed_from.artifact_sha256`
- `identity_explain.composed_from.path`

## SOURCE: docs/spec/ATTEMPT_TRACE_SCHEMA_V0.1.md
# Attempt Trace Schema v0.1

Attempt trace records are distinct from node summary status.

## Required fields

- `node_id` (string)
- `attempt` (integer >= 1)
- `backend_kind` (string)
- `status` (string: success|failed|skipped|cached|cancelled)
- `exit_code` (integer|null)

## Compatibility

- Additive fields allowed in minor updates.
- Existing required fields are stable within `v0.1`.

## Owner

- Runtime execution backend contract.

## SOURCE: docs/spec/BUNDLE_MANIFEST_VERSIONING_POLICY.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/BUNDLE_MANIFEST_VERSIONING_POLICY.md](./appendices/versioning/BUNDLE_MANIFEST_VERSIONING_POLICY.md)

## SOURCE: docs/spec/BUNDLE_SCHEMA_REFERENCE.md
# Bundle Schema Reference

This page indexes bundle format references and fixture examples.

## Format specs

- `docs/spec/GRAPH_BUNDLE_FORMAT_V1.md`
- `docs/spec/RUN_BUNDLE_FORMAT_V1.md`
- `docs/spec/ARTIFACT_BUNDLE_FORMAT_V1.md`
- `docs/spec/BUNDLE_MANIFEST_VERSIONING_POLICY.md`

## Fixture examples

- Minimal bundle: `evidence/compat/export_bundle/v0_1_supported/examples/minimal_bundle.json`
- Maximal bundle: `evidence/compat/export_bundle/v0_1_supported/examples/maximal_bundle.json`

## Verification surfaces

- `bijux dag import --verify-only <bundle>`
- `bijux dag fsck <bundle> --json`

## SOURCE: docs/spec/COMPATIBILITY_PROMISE.md
# Compatibility Promise

## Scope
Defines compatibility commitments for pre-release and stable tracks.

## Tracks
- `0.x` pre-release: breaking changes may occur, but must be documented and migration-noted.
- `1.x+` stable: breaking changes require explicit major version increment and compatibility notes.

## Support window
Supported compatibility window is documented in `docs/COMPATIBILITY.md`.

## Related tests
- `configs/schema/fixtures/compat/positive/*`
- `configs/schema/fixtures/compat/negative/*`

## Versioning and change policy
Support-window changes require release-policy update and compatibility matrix refresh.

## SOURCE: docs/spec/CONFIG_DEPRECATION.md
# Superseded by config cluster contract

- Superseded by: [CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md](./CONFIG_AND_STATE_BOUNDARIES_CONTRACT.md)
- Appendix source: [appendices/config/CONFIG_DEPRECATION.md](./appendices/config/CONFIG_DEPRECATION.md)

## SOURCE: docs/spec/NODE_TRACE_SCHEMA_V0.1.md
# Node Trace Schema v0.1

Source of truth schema: `configs/schema/node_trace.schema.json`.

Required keys:
- `node_id` (string)
- `status` (string: success|failed|skipped|cached)
- `started_unix_ms` (number)
- `finished_unix_ms` (number)
- `attempt` (number)
- `fingerprint` (string)
- `adapter_id` (string)
- `adapter_version` (string)

Optional:
- `adapter_binary_sha256` (string)
- `resources` (object)
- `inputs_index` (string)
- `resolved_params` (json)
- `container` (object)
- `cache_proof` (object)
- `skip_reason` (object)
- `failure` (object)

## cache_proof
```
{
  "hit": bool,
  "key": "string",
  "source": "local|remote|pack",
  "verified": bool,
  "reason": "string",
  "corrupt_detected": bool
}
```

## failure
```
{
  "kind": "Validation|Execution|Timeout|Cancelled|CacheCorrupt|Internal",
  "code": "string",
  "message": "string",
  "details": <json>?
}
```

## skip_reason
```
{
  "reason": "string"
}
```

## container
```
{
  "image": "string",
  "image_digest": "string?",
  "engine": "docker|podman",
  "engine_version": "string?",
  "exit_code": "number?"
}
```

## SOURCE: docs/spec/OUTPUTS_INDEX_SCHEMA_V0.1.md
# Outputs Index Schema v0.1

Source of truth schema: `configs/schema/outputs_index.schema.json`.

Required keys:
- `files` (array)

Each file item requires:
- `path` (string)
- `sha256` (string)
- `node_id` (string)
- `node_fingerprint` (string)

## SOURCE: docs/spec/REPLAY_PROOF_BUNDLE_SCHEMA_V0.1.md
# Replay Proof Bundle Schema v0.1

Replay proof output for `dag replay --prove --json` is defined by:

- `configs/schema/operator/replay_proof.schema.json`

Required fields:

- `fidelity_level`
- `equivalent`
- `reasons`
- `reason_report`
- `cause_groups`
- `source_run_id`
- `replay_run_id`


## SOURCE: docs/spec/RUNTIME_TELEMETRY_SCHEMA.md
# Runtime Telemetry Schema

## Purpose

Define a stable runtime telemetry envelope for operator diagnostics and release verification.

## Canonical schema

- `configs/schema/operator/runtime_telemetry.schema.json`
- schema version: `runtime-telemetry/v0.1`

## Required coverage signals

- node-duration telemetry
- run-duration telemetry
- scheduler telemetry
- cache hit and miss telemetry
- replay telemetry
- diff telemetry
- prove/verify telemetry
- artifact IO telemetry
- backend capability telemetry
- failure, cancellation, and partial-rerun telemetry

## Compatibility guarantees

- stable required keys are backward compatible within `v0.1.x`
- forward schema versions are rejected until explicitly supported

## SOURCE: docs/spec/RUN_MANIFEST_SCHEMA_V0.1.md
# Run Manifest Schema v0.1

Source of truth schema: `configs/schema/run_manifest.schema.json`.

Required keys:
- `run_id` (string)
- `created_unix_ms` (number)
- `started_unix_ms` (number)
- `finished_unix_ms` (number)
- `graph_snapshot` (string)
- `graph_fingerprint` (string)
- `status` (string: success|failed|cancelled)
- `spec` (string)
- `tool_version` (string)
- `jobs` (number)
- `adapters` (array)
- `node_counts` (object)
- `policy` (object)

Optional:
- `run_timeout_ms` (number)
- `cache_mode` (string)
- `cache_dir` (string)
- `outputs` (array)

## SOURCE: docs/spec/RUN_SUMMARY_SCHEMA_V0.1.md
# Run Summary Schema v0.1

The run summary object is emitted in run manifests under `run_summary`.

Fields:

- `total_nodes` (u32)
- `success` (u32)
- `failed` (u32)
- `skipped` (u32)
- `cached` (u32)

These counters are advisory aggregation surfaces and do not replace per-node trace truth.

## SOURCE: docs/spec/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md](./appendices/versioning/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md)

## SOURCE: docs/spec/SCHEMA_COMPATIBILITY_POLICY.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/SCHEMA_COMPATIBILITY_POLICY.md](./appendices/versioning/SCHEMA_COMPATIBILITY_POLICY.md)

## SOURCE: docs/spec/SCHEMA_EVOLUTION_POLICY.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/SCHEMA_EVOLUTION_POLICY.md](./appendices/versioning/SCHEMA_EVOLUTION_POLICY.md)

## SOURCE: docs/spec/SCHEMA_EVOLUTION_RULEBOOK.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/SCHEMA_EVOLUTION_RULEBOOK.md](./appendices/versioning/SCHEMA_EVOLUTION_RULEBOOK.md)

## SOURCE: docs/spec/SCHEMA_FIELD_DEPRECATION_POLICY.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/SCHEMA_FIELD_DEPRECATION_POLICY.md](./appendices/versioning/SCHEMA_FIELD_DEPRECATION_POLICY.md)

## SOURCE: docs/spec/SCHEMA_FORWARD_COMPATIBILITY_LIMITATIONS.md
# Schema Forward-Compatibility Limitations

- Future schema versions are rejected unless explicitly supported.
- Unknown required fields in stable schema payloads are treated as incompatibility.
- Forward compatibility is not implied by additive draft fields.

## SOURCE: docs/spec/STABLE_EXPERIMENTAL_SCHEMA_FIELDS.md
# Stable and Experimental Schema Fields

## Stable fields
- DAG: `spec`, `nodes[].id`, `nodes[].command`
- Run manifest: `manifest_version`, `graph_fingerprint`, `status`
- Outputs index: `files[].path`, `files[].sha256`
- Proof bundle: `schema_version`, `proof_id`, `run_id`, `status`

## Experimental fields
- Run manifest: `backend_metadata` (experimental)
- Proof bundle: `signing.signature_format` (experimental)
- Proof bundle: `signing.signature` (experimental)
- Capability matrix: `semantic_portability` (experimental)

## SOURCE: docs/spec/STABLE_SCHEMA_COMPATIBILITY_REVIEW_CHECKLIST.md
# Stable Schema Compatibility Review Checklist

1. Confirm schema authority path under `configs/schema/` is correct and versioned.
2. Confirm command-family mapping is declared in `configs/policy/json_output_governance.json`.
3. Confirm minimal and maximal examples exist under `evidence/operator/examples/stable_json/<schema>/`.
4. Confirm lockstep test marker exists in `crates/bijux-dev-dag/tests/json_output_governance_contracts.rs`.
5. Confirm generated inventories are refreshed:
- `docs/reports/foundation/JSON_COMMAND_SCHEMA_INVENTORY_REPORT.md`
- `docs/reports/foundation/SCHEMA_COMMAND_TEST_INVENTORY_REPORT.md`
6. Confirm gap reports stay zero:
- `docs/reports/foundation/schema_without_example_output_report.md`
- `docs/reports/foundation/commands_without_json_lockstep_report.md`
7. Confirm schema registry and stable command registry pages are refreshed.
8. Confirm release gate suite still includes JSON output governance contract tests.

## SOURCE: docs/spec/UNIFIED_SCHEMA_VERSIONING_POLICY.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/UNIFIED_SCHEMA_VERSIONING_POLICY.md](./appendices/versioning/UNIFIED_SCHEMA_VERSIONING_POLICY.md)

## SOURCE: docs/spec/VERSIONING.md
# Superseded versioning index

This document is now an archived index.

Canonical source: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)  
Source appendix: [appendices/versioning/VERSIONING.md](./appendices/versioning/VERSIONING.md)

## SOURCE: docs/spec/VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md
# Versioning and schema evolution contract

**What this spec is not**: release note prose, operational onboarding, or evidence dashboards.

## Scope

This contract is the authoritative source for all versioned contracts in the project:

- binary / crate compatibility and release compatibility decisions
- schema families (`dag`, `run`, `artifact`, `proof`, `bundle`, `evidence`, `operator`)
- migration obligations and compatibility windows
- deprecation and compatibility drift controls

## Stable contract set

1. [Versioning model and stability](./appendices/versioning/VERSIONING_MODEL.md)
2. [Schema compatibility and evolution policy](./appendices/versioning/SCHEMA_EVOLUTION_POLICY.md)
3. [Bundle manifest migration and versioning](./appendices/versioning/BUNDLE_MANIFEST_VERSIONING_POLICY.md)
4. [Unified surface compatibility policy](./appendices/versioning/UNIFIED_SCHEMA_VERSIONING_POLICY.md)
5. [Configuration deprecation policy](./appendices/versioning/SCHEMA_FIELD_DEPRECATION_POLICY.md)

## Core rules

- Stable schema contracts only change through explicit compatibility policy and migration evidence.
- Additive fields are allowed when defaults preserve behavior and do not change canonical meaning.
- Breaking shape changes require version bump and migration evidence.
- Compatibility decisions and drift controls must be synchronized with contract tests and evidence suites.

## Drift and migration controls

- Version compatibility drift is rejected unless documented in compatibility artifacts and fixtures.
- Forward-incompatible versions are rejected with deterministic diagnostics until support is explicitly added.
- Schema and manifest migration behavior must be deterministic, deterministic, and side-effect safe.

## Evidence and implementation links

- Source-of-truth: `configs/schema/` family schemas and `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`.
- Required evidence: `evidence/compat/`, compatibility fixtures, migration suites, and schema changelog.
- Regression signals: governance suites under `crates/bijux-dev-dag` and evidence reports.

## Superseded-by mapping

- Temporary artifacts retained in `docs/spec/appendices/versioning/` include moved historical versions.
- Canonical migration references from previous naming remain:
  - `VERSIONING.md`
  - `VERSIONING_MODEL.md`
  - `UNIFIED_SCHEMA_VERSIONING_POLICY.md`
  - `SCHEMA_COMPATIBILITY_POLICY.md`
  - `SCHEMA_EVOLUTION_POLICY.md`
  - `SCHEMA_EVOLUTION_RULEBOOK.md`
  - `SCHEMA_FIELD_DEPRECATION_POLICY.md`
  - `SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md`
  - `VERSION_COMPATIBILITY_DRIFT_POLICY.md`
  - `BUNDLE_MANIFEST_VERSIONING_POLICY.md`

## SOURCE: docs/spec/VERSIONING_MODEL.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/VERSIONING_MODEL.md](./appendices/versioning/VERSIONING_MODEL.md)

## SOURCE: docs/spec/VERSION_COMPATIBILITY_DRIFT_POLICY.md
# Superseded by versioning contract

- Superseded by: [VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md](./VERSIONING_AND_SCHEMA_EVOLUTION_CONTRACT.md)
- Appendix source: [appendices/versioning/VERSION_COMPATIBILITY_DRIFT_POLICY.md](./appendices/versioning/VERSION_COMPATIBILITY_DRIFT_POLICY.md)

## SOURCE: docs/spec/appendices/config/CONFIG_DEPRECATION.md
# Config Deprecation Policy

## Scope
Defines how configuration fields are deprecated and removed.

## Current status
No config fields are currently deprecated.

## Rules
- Deprecated fields must be explicitly listed in this document with replacement guidance.
- Deprecated fields remain accepted only for a documented compatibility window.
- New deprecated fields require matching validation tests and migration notes.

## Related tests
- `crates/bijux-dag-app/tests/config_validation_contract.rs`

## Versioning and change policy
Deprecation additions are contract changes and must be reviewed with config precedence and schema compatibility docs.

## SOURCE: docs/spec/appendices/versioning/BUNDLE_MANIFEST_VERSIONING_POLICY.md
# Bundle Manifest Versioning and Migration Policy

## Scope

Defines version evolution for graph/run/artifact bundle formats.

## Current supported bundle version

- `export-bundle/v0.1`

## Compatibility rules

- Backward compatibility is required for all supported prior bundle versions.
- Unsupported bundle versions must fail import with explicit diagnostics.
- Migration behavior must be deterministic and idempotent.

## Migration policy

- `import --verify-only` must run structural and invariant checks without mutating runtime records.
- Migration tooling may add non-semantic metadata but must not alter canonical identities promised by contract.

## Release governance

- Release verification must include bundle conformance suites and backward-compatibility fixture checks.

## SOURCE: docs/spec/appendices/versioning/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md
# Schema Backward-Compatibility Guarantees

- Supported versions are listed in `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`.
- Stable schemas accept previously supported payloads in the compatibility window.
- Unsupported past versions fail with explicit diagnostics.

## SOURCE: docs/spec/appendices/versioning/SCHEMA_COMPATIBILITY_POLICY.md
# Schema compatibility policy

Schemas under `configs/schema/` are the source of truth for wire contracts.

## Compatibility classes
- Additive: adding optional fields, expanding enums with backward-safe defaults, adding optional objects.
- Breaking: removing fields, changing field types, making optional fields required, narrowing enums, changing required semantics.

## Versioning
- Breaking changes require a new schema version and migration notes.
- Additive changes remain within the same version only when old clients continue to parse and operate correctly.

## Fixture policy
- Each schema version must include positive and negative fixtures.
- Negative fixtures must include unknown fields, invalid enum values, malformed references, and invalid path shapes.

## SOURCE: docs/spec/appendices/versioning/SCHEMA_EVOLUTION_POLICY.md
# Schema Evolution Policy

## Intent

Maintain durable operator and machine interfaces while allowing controlled evolution.

## Compatibility rules

- Backward compatibility is required for all stable schema families in `v0.1.x`.
- Forward schema versions are rejected with explicit diagnostics until officially supported.
- Optional additive fields are allowed when defaults preserve prior semantics.
- Breaking shape changes require a schema version bump and migration fixtures update.

## Stable families

- Graph schema (`configs/schema/dag.schema.json`)
- Run schema (`configs/schema/run_manifest.schema.json`)
- Artifact schema (`configs/schema/outputs_index.schema.json`)
- Proof schema (`docs/spec/PROOF_BUNDLE_SCHEMA_v0.1.json`)
- Operator diff/explain surfaces under `configs/schema/operator/`

## Required evidence for schema changes

- Backward and forward compatibility fixtures in `evidence/compat/`.
- Migration source fixtures for the oldest supported version in `evidence/compat/migrations/`.
- Compatibility matrix refresh.
- Schema changelog refresh.

## Validation requirements

- Migration behavior must remain deterministic and idempotent.
- Compatibility diagnostics must remain machine-readable and stable.

## SOURCE: docs/spec/appendices/versioning/SCHEMA_EVOLUTION_RULEBOOK.md
# Schema Evolution Rulebook

## Additive changes
Allowed when new fields are optional and defaults preserve existing semantics.

## Deprecation changes
Allowed when deprecated fields retain behavior and are documented with migration guidance.

## Breaking changes
Require graph schema version bump, compatibility fixture updates, and explicit release note entry.

## SOURCE: docs/spec/appendices/versioning/SCHEMA_FIELD_DEPRECATION_POLICY.md
# Schema Field and Command Deprecation Policy

## Field deprecation
- Deprecations must include replacement guidance.
- Deprecated fields remain parseable within the compatibility window.
- Removal requires version bump and compatibility fixture updates.

## Command deprecation
- CLI deprecations follow `docs/spec/CLI_DEPRECATION_AND_ALIAS_POLICY.md`.
- Stable command removal requires explicit release-note migration guidance.

## SOURCE: docs/spec/appendices/versioning/UNIFIED_SCHEMA_VERSIONING_POLICY.md
# Unified Schema Versioning Policy

## Scope
This policy governs versioning for graph, run, artifact, and proof schemas.

## Versioned surfaces
- graph schema: `configs/schema/dag.schema.json`
- run manifest schema: `configs/schema/run_manifest.schema.json`
- artifact/index schemas: `configs/schema/outputs_index.schema.json`, `configs/schema/node_trace.schema.json`
- proof schema: `docs/spec/PROOF_BUNDLE_SCHEMA_v0.1.json`

## Stability classes
- stable: backward-compatible within declared support window
- experimental: may change before promotion to stable

## Change rules
- breaking changes require version bump and migration notes
- additive stable changes require compatibility fixtures and contract tests
- experimental fields must be explicitly labeled `(experimental)` in docs

## Operator inspection and migration surfaces
- schema inspect: `bijux dag version-inspect`
- schema migration: `bijux dag migrate dag|run [--dry-run]`

## SOURCE: docs/spec/appendices/versioning/VERSIONING.md
# Versioning

Normative versioning policy moved to:
- `docs/spec/VERSIONING_MODEL.md`
- `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`
- `docs/spec/MIGRATION_POLICY.md`

This file remains as a compatibility pointer only.

## SOURCE: docs/spec/appendices/versioning/VERSIONING_MODEL.md
# Versioning Model

## Versioned surfaces
- binary version: CLI/package version (`cargo` semver)
- crate API version: per crate semver and stability scope
- graph schema version: DAG `spec` field
- run-dir format version: manifest `manifest_version`
- export bundle version: bundle `export_bundle_version`

## Compatibility matrix authority
See `docs/reference/SUPPORT_AND_COMPATIBILITY_MATRICES.md`.

## Compatibility rules
- Additive schema fields: allowed if defaults preserve behavior.
- Deprecations: must include docs + fixture coverage.
- Breaking changes: require explicit version bump and negative fixtures.
- Silent reinterpretation of unsupported versions is forbidden.

## SOURCE: docs/spec/appendices/versioning/VERSION_COMPATIBILITY_DRIFT_POLICY.md
# Version Compatibility Drift Policy

No silent compatibility drift is allowed.
Any change to versioned surfaces must update:
- compatibility fixtures
- compatibility matrix docs
- control-plane compatibility checks
