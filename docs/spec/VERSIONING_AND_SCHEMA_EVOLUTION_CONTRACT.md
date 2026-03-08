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

- Source-of-truth: `configs/schema/` family schemas and `docs/reference/COMPATIBILITY_MATRIX.md`.
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
