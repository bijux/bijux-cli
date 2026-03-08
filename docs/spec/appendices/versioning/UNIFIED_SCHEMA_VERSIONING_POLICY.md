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
