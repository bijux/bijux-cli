# Control Plane Service Boundary

Status: accepted
Owner: dev control-plane maintainers
Date: 2026-03-09

## Decision
`bijux-dev-dag` is the single control-plane automation surface for governance workflows. Root scripts and duplicated automation entrypoints are disallowed.

## Consequences
- Governance automation remains centralized and auditable.
- Command-surface growth is controlled by freeze rules.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260309-CONTROL-PLANE-SERVICE-MIGRATION-BOUNDARY.md
# Control plane migration boundary

## Server crate placeholder

Planned crate name: `bijux-dag-api`.

Purpose:

- host service control-plane endpoints
- integrate persistent registry and scheduler backends
- enforce authorization and policy decisions

## What remains in CLI now

- repository-oriented quality checks
- local compatibility and contract execution
- local schedule validation and preview
- local artifact and observability reporting

## What migrates to service control plane later

- shared DAG registry publication workflow
- remote schedule persistence and trigger evaluation
- multi-user run-control operations
- policy bundle distribution and decision endpoints
- organization-scoped authorization decisions

## Compatibility intent

`bijux-dev-dag` keeps stable command semantics while transport and persistence move behind `bijux-dag-api`.

### SOURCE: 20260308-DEV-DAG-GOVERNANCE-SCOPE.md
# ADR: dev-dag Governance Scope

## Status
Accepted

## Context
`bijux-dev-dag` accumulated mixed concerns, including checks that risked becoming authoritative for runtime semantics.

## Decision
`bijux-dev-dag` remains governance and verification orchestration only:
- Repository policy checks and generated governance reports.
- Contract and release checks that validate other crates.
- No authoritative ownership of runtime execution semantics or schema definitions.

## Consequences
- Runtime/core/artifacts keep source-of-truth ownership.
- Governance automation remains strict but non-authoritative.
- Drift checks can enforce scope boundaries with clear failure modes.

### SOURCE: 20260308-DEV-DAG-COMMAND-DECOMPOSITION-SHAPE.md
# ADR: Dev-Dag Command Decomposition Shape

- Date: 2026-03-08
- Status: Accepted

## Context

`crates/bijux-dev-dag/src/commands/mod.rs` accumulated broad orchestration, filesystem traversal, and command dispatch responsibilities. This reduced readability and made direct ownership of command families less explicit.

## Decision

- Keep `commands/mod.rs` as the primary command dispatch surface.
- Extract reusable file traversal and run-selection helpers into `commands/file_catalog.rs`.
- Keep command-family business logic in focused modules (`authoring_evidence`, `battle_evidence`, `compare_evidence`, `evidence_control_plane`, `evidence_registry`, `perf_evidence`, `suite_catalog`).
- Require direct test surfaces in each command-family module and verification binary.
- Enforce release-time 0%-coverage guardrails via dev-dag contract tests and protected allowlist checks.

## Consequences

- Command ownership is clearer and easier to review.
- Changes to filesystem traversal logic are isolated in one helper module.
- Direct tests remain close to the command logic and binaries they validate.
- Further reductions of `commands/mod.rs` can continue with incremental helper extractions without changing command-line behavior.
