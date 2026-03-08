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
