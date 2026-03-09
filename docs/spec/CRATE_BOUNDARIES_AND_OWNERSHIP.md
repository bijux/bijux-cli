# CRATE BOUNDARIES AND OWNERSHIP

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/CRATE_API_POLICY.md
# Crate API policy

## Public visibility defaults
- Default visibility is `pub(crate)`.
- `pub` is allowed only for documented crate contracts.
- New public exports require corresponding crate documentation updates.

## CLI boundary
- `bijux-dag-cli` is a thin binary crate only.
- Business logic is forbidden in `bijux-dag-cli`.

## Core boundary
`bijux-dag-core` exports stable model, parse, resolve, validation, topology, and fingerprint API surfaces.
Core exports must remain deterministic and side-effect free.

## Runtime boundary
`bijux-dag-runtime` exports runtime API/config/policy/result/context surfaces and keeps adapter execution internals behind module boundaries.

## App boundary
`bijux-dag-app` exports orchestration entrypoints and keeps clap command model and rendering internals module-scoped.

## Dev boundary
`bijux-dev-dag` exports repository governance command surfaces and enforces workspace contracts.

## Artifact boundary
`bijux-dag-artifacts` is an artifact model and persistence API crate.
- It may own artifact storage operations through stable APIs.
- Runtime may interact with artifact persistence only through public artifact APIs.

## Formatting helper reuse
No crate may depend on another crate solely to reuse rendering or formatting helpers.
Shared rendering helpers must live in the consuming crate or a dedicated neutral utility crate.

## Adapter placement
Built-in adapters remain in `bijux-dag-runtime` for now.
- Adapter contracts and type-level boundaries are exposed through `runtime::adapter_api`.
- Adapter-specific execution logic must not leak across unrelated runtime modules.

## SOURCE: docs/spec/CRATE_BOUNDARY_CONTRACT.md
# Crate Boundary Contract

## Forbidden edges
- `bijux-dag-runtime -> bijux-dag-app`
- `bijux-dag-runtime -> bijux-dag-cli`
- `bijux-dag-core -> bijux-dag-runtime`
- `bijux-dag-core -> bijux-dag-app`

## Thin CLI rule
`bijux-dag-cli` is dispatch-only and must not implement execution, scheduling, or artifact semantics.

## App orchestration rule
`bijux-dag-app` may orchestrate runtime calls and output shaping, but must not contain scheduler internals.

## Runtime policy
Runtime owns execution semantics and consumes core/artifact services through typed interfaces.

## SOURCE: docs/spec/CRATE_OWNERSHIP.md
# Crate ownership and domain authority

## Domain map

- `bijux-dag-core`: model
- `bijux-dag-artifacts`: artifacts
- `bijux-dag-runtime`: execution
- `bijux-dag-app`: app orchestration
- `bijux-dag-cli`: CLI
- `bijux-dag-testkit`: shared test helpers and fixtures
- `bijux-dev-dag`: repo governance

## Public module contract

The enforceable contract is `configs/policy/crate_ownership.json`.

`bijux-dev-dag` validates that each crate exports only declared public modules.

## SOURCE: docs/spec/CRATE_OWNERSHIP_MATRIX.md
# Crate Ownership Matrix

## Runtime

- ownership: runtime execution kernel and adapter/runtime boundaries
- not owned: CLI routing, release evidence authority, schema source-of-truth ownership

## Core

- ownership: graph model, canonicalization, validation semantics
- not owned: runtime execution backend behavior or adapter implementations

## Artifacts

- ownership: artifact identity, lineage, storage/transport primitives
- not owned: command routing and CLI argument parsing

## App

- ownership: command routing, operator UX surfaces, output rendering
- not owned: low-level artifact storage primitives

## Dev Governance

- ownership: release evidence, policy checks, governance reports
- not owned: authoritative runtime execution semantics

## SOURCE: docs/spec/CRATE_RESPONSIBILITY_ALIGNMENT.md
# Crate Responsibility Alignment

This document aligns current code ownership boundaries with runtime contraction governance.

## Aligned Responsibilities

- `bijux-dag-core`: graph identity, canonicalization, validation.
- `bijux-dag-runtime`: execution engine, scheduler, runtime adapter boundaries.
- `bijux-dag-artifacts`: artifact identity, lineage, storage interfaces.
- `bijux-dag-app`: command routing and operator UX.
- `bijux-dev-dag`: release evidence and governance checks.

## Drift Handling

- boundary drift is blocked by crate responsibility guardrails contracts.

## SOURCE: docs/spec/CRATE_RESPONSIBILITY_STATEMENTS.md
# Crate Responsibility Statements

Authoritative taxonomy policy lives in `configs/policy/crate_taxonomy_v2.json`.

## `bijux-dag-core`
- DAG schema, parsing, canonicalization, validation, and semantic graph rules.
- No CLI or runtime execution orchestration.

## `bijux-dag-artifacts`
- Run directory schemas, artifact models, path normalization, integrity metadata, retention helpers.
- No scheduler or node execution policy logic.

## `bijux-dag-runtime`
- Execution engine, scheduler semantics, state transitions, policy enforcement during execution.
- Consumes core + artifacts contracts, but does not define CLI UX.

## `bijux-dag-app`
- Command orchestration, user-facing command output shaping, inspect/verify command behavior.
- No scheduler internals or adapter execution implementation.

## `bijux-dag-cli`
- Thin binary wrapper over app command tree.
- No runtime semantics.

## `bijux-dag-testkit`
- Shared workspace test helpers and fixture utilities.

## `bijux-dev-dag`
- Repository control-plane checks, governance suites, drift and boundary verification.

## SOURCE: docs/spec/CRATE_TAXONOMY_V2.md
# Crate taxonomy v2

## Purpose

Define stable workspace crate ownership and dependency boundaries for the current foundation scope.

## One-sentence responsibilities

- `bijux-dag-core`: DAG schema, parsing, canonicalization, validation, and deterministic semantic graph logic.
- `bijux-dag-artifacts`: run artifact models, persistence services, integrity proofs, and lifecycle policy helpers.
- `bijux-dag-runtime`: execution engine, scheduler behavior, policy enforcement, replay semantics, and runtime diagnostics.
- `bijux-dag-testkit`: shared deterministic test fixtures, builders, and assertion helpers for workspace crates.
- `bijux-dag-app`: application orchestration services, command response modeling, and user-facing render flows.
- `bijux-dag-cli`: thin process entrypoint that delegates to app command surfaces.
- `bijux-dev-dag`: repository governance control-plane, suite orchestration, and release verification automation.

## Dependency boundary

Allowed workspace edges are defined in:

- `configs/policy/crate_taxonomy_v2.json`

`bijux-dev-dag` enforces this policy through taxonomy guardrail tests.

## Taxonomy decisions

- app remains one crate for this scope.
- artifacts remains one crate with explicit internal sub-boundaries.
- planning remains in core with runtime bridge consumption.
- runtime remains one crate after contraction and policy freeze.
- testkit remains shared support for tests.
- container/remote/batch stay modeled in runtime as future execution boundaries.

## Stability rule

This taxonomy is in frozen mode. New workspace crates are blocked until this document and `crate_taxonomy_v2` policy are explicitly revised together.

## SOURCE: docs/spec/KERNEL_ALLOWED_DEPENDENCIES.md
# Kernel Allowed Dependencies

This document defines dependency rules for the deterministic kernel path.

## Scope

Kernel scope is defined by:

- `docs/spec/KERNEL_BOUNDARY_CONTRACT.md`
- `configs/policy/kernel_dependency_policy.json`

## Core crate (`bijux-dag-core`) allowed dependencies

- `serde`
- `serde_json`
- `sha2`
- `hex`
- `thiserror`
- `criterion` (dev-only benchmark dependency)
- `tempfile` (dev-only test dependency)

## Runtime kernel path disallowed dependency classes

- CLI parsing libraries (`clap`)
- HTTP/server frameworks (`axum`, `warp`)
- repository/network governance clients (`git2`, `octocrab`, `reqwest`)
- app/dev crates (`bijux-dag-app`, `bijux-dev-dag`)

## Enforcement

- `crates/bijux-dev-dag/tests/no_runtime_in_core.rs`
- `crates/bijux-dev-dag/tests/runtime_contraction_contracts.rs`
- `crates/bijux-dev-dag/tests/dependency_boundary_contracts.rs`

## SOURCE: docs/spec/KERNEL_BOUNDARY_CONTRACT.md
# Kernel Boundary Contract

## Scope

Defines the conceptual kernel boundary for bijux-dag.

## Kernel definition

Kernel means the deterministic execution truth path:

- canonical graph parsing and validation
- execution planning
- scheduler readiness and ordering
- run-state transitions
- artifact commit and lineage identity
- replay and diff semantic verification

## Modules in kernel ownership

- `crates/bijux-dag-core/src/graph/**`
- `crates/bijux-dag-core/src/pipeline/**`
- `crates/bijux-dag-core/src/planner/**`
- `crates/bijux-dag-core/src/analysis/fingerprint.rs`
- `crates/bijux-dag-runtime/src/runtime_core/**`
- `crates/bijux-dag-runtime/src/artifacts/**`
- `crates/bijux-dag-runtime/src/cache/**`
- `crates/bijux-dag-runtime/src/replay/**`
- `crates/bijux-dag-runtime/src/policy/**`

## Modules excluded from kernel ownership

- CLI route, rendering, and dev governance surfaces.
- Runtime modeled/future platform surfaces (`internal/**`, `backend/distributed/**`, `backend/runtime/*execution*` except stable local path semantics).
- AI/operator-assist and control-plane reporting modules.
- Evidence report generation and release-report formatting modules.

## Dependency invariants

- Kernel code must not depend on CLI crates.
- Kernel code must not depend on dev governance crates.
- Kernel code must not read or format evidence report content.

## Related reports

- `docs/reports/foundation/KERNEL_API_SURFACE_REPORT.md`
- `docs/reports/foundation/PUBLIC_API_SHRINK_REPORT.md`

## SOURCE: docs/spec/KERNEL_DEPENDENCY_POLICY.md
# Kernel Dependency Policy

## Purpose

Defines allowed and forbidden dependency classes for kernel-owned code.

## Allowed classes

- `serde`, `serde_json`
- hashing and deterministic canonicalization dependencies
- core/runtime artifact and cache dependencies
- deterministic scheduler/planner dependencies

## Forbidden classes

- CLI and command-line parsing dependencies in kernel crates
- dev governance/control-plane dependencies in kernel crates
- report-generation and evidence-report formatting dependencies in kernel crates

## Machine policy source

- `configs/policy/kernel_dependency_policy.json`

## Related tests

- `crates/bijux-dev-dag/tests/kernel_boundary_contracts.rs`
