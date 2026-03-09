# WORKSPACE PROJECT AND API CONTRACTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/API_CONTRACT.md
# API resource contract draft

This document defines the minimal service control-plane resource model for the future `dag-api`.

## Resources

- DAGs
- DAG versions
- runs
- node attempts
- artifacts
- schedules
- queues
- policies
- audit events

## Control-plane operations

- registry: publish, validate, activate, deprecate, retire, inspect
- run-control: submit, cancel, pause, resume, retry, replay, verify
- artifact: inspect, export, verify, lineage, retention action
- schedule: create, update, suspend, preview, audit

## Request/response and list contracts

- typed request envelope: `TypedApiRequest`
- typed response envelope: `TypedApiResponse`
- pagination: `Pagination { limit, cursor }`
- filtering: `ListFilter { field, value }`
- mutable versioning: `VersionedResource { resource_version, etag }`

## Authentication and authorization boundaries

- principals: CLI user, service account, worker identity
- authorization: scoped action evaluation via typed rules

## Environment and subscription contracts

- environment-scoped configuration uses typed values + overlays
- event subscription model is typed for future webhooks/streaming

## Storage and reproducibility

- DAG registry storage is abstracted for filesystem and database implementations.
- Policy bundles are versioned to make decisions reproducible.
- Schedule definitions are separated from execution submissions.

## API compatibility and evolution

- API versions are explicit (`major`, `minor`)
- major versions must satisfy compatibility bounds
- minor versions are additive-only

## CLI compatibility mapping

Current `bijux-dev-dag` commands map to future service operations as follows:

- `checks run` -> repository validation endpoint
- `contracts run` -> contract execution endpoint
- `schedule validate` -> schedule compile endpoint
- `schedule preview` -> schedule simulation endpoint
- `observability-report` -> run observability report endpoint

The mapping keeps command semantics stable when CLI becomes a thin service client.

## SOURCE: docs/spec/PROJECT_CONTRACT.md
# Project Contract

## Scope
Defines product-level goals, non-goals, stability, and compatibility constraints for the DAG engine.

## Goals
- Provide a strict, minimal DAG IR.
- Ensure deterministic execution order.
- Produce reproducible run artifacts.

## Non-Goals
- Distributed execution.
- Dynamic graph mutation at runtime.
- Implicit network access.

## Compatibility
- Spec versions live in `spec/`.
- Breaking changes require a new version file.

## Stability
- JSON parsing is strict (`deny_unknown_fields`).
- Canonical output uses stable ordering.

## Related tests
- `crates/bijux-dag-core/tests/contract_stability.rs`
- `crates/bijux-dag-core/tests/canonicalization_ordering.rs`

## Versioning and change policy
Project-level contract changes are additive by default; breaking scope or compatibility changes require explicit contract version note and linked evidence updates.

## SOURCE: docs/spec/SELECTOR_CONTRACT.md
# Selector Contract

## Scope
Defines selection and exclusion semantics for node execution targeting.

## Invariants
- Selection filtering is deterministic.
- Include and exclude interaction rules are explicit and stable.
- Invalid selector references fail validation before execution.

## Related tests
- `evidence/battle/workflows/selection/*`
- `crates/bijux-dag-runtime/tests/selector_filtering_contract.rs`

## Related schemas
- `configs/schema/dag.schema.json`

## Versioning and change policy
Selector semantic changes require compatibility review and updated integration coverage.

## SOURCE: docs/spec/WORKSPACE_CONTRACT.md
# Workspace contract

This document defines crate responsibilities and allowed dependency directions.
Taxonomy authority is defined in `configs/policy/crate_taxonomy_v2.json` and `docs/spec/CRATE_TAXONOMY_V2.md`.

## Scope
Defines workspace crate responsibility boundaries and allowed dependency directions.

## Crate responsibilities

- `bijux-dag-core`: DAG model, parsing, canonicalization, validation, fingerprinting, and topology algorithms. Pure logic only.
- `bijux-dag-artifacts`: artifact/run manifest data models plus artifact persistence contracts (`format + IO`).
- `bijux-dag-runtime`: execution planning/runtime, scheduling flow, adapter invocation boundaries, policy enforcement, and trace emission.
- `bijux-dag-app`: application orchestration commands and structured output rendering.
- `bijux-dag-cli`: binary wiring and process-level error mapping only.
- `bijux-dev-dag`: repository governance, contract checks, release verification orchestration.

## Allowed crate dependency directions

- `bijux-dag-core`: may not depend on runtime/app/cli/dev crates.
- `bijux-dag-artifacts`: may depend on core models only when required by artifact contracts.
- `bijux-dag-runtime`: may depend on core and artifacts.
- `bijux-dag-app`: may depend on core, artifacts, runtime.
- `bijux-dag-cli`: may depend on app only.
- `bijux-dev-dag`: must not depend on runtime internals or app runtime orchestration internals.

## Enforcement

Boundary policy is enforced by `configs/policy/dependency_rules.json` through `bijux-dev-dag dep-guard`.

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs` (`run_dep_guard`, manifest guards)
- `crates/bijux-dev-dag/src/suites/repo.rs`

## Versioning and change policy
Dependency direction changes require coordinated updates to policy JSON, this contract, and repo governance checks.
