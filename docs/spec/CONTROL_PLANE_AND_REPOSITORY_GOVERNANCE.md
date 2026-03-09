# CONTROL PLANE AND REPOSITORY GOVERNANCE

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/CONTROL_PLANE_FOUNDATION.md
# Control-plane foundation

## Scope

`bijux-dev-dag` is the source of truth for repository verification, release verification orchestration, and governance suite execution.

## Required command surfaces

- repo verification
- docs verification
- naming verification
- crate boundary verification
- fixture verification
- artifact contract verification
- release verification
- foundation hardening verification
- ci verification

## Required guard surfaces

- root directory guard
- executable guard
- config verification
- schema verification
- docs taxonomy verification
- test trust verification
- runtime taxonomy verification

## Foundation suite

Suite id `foundation-verification` validates that core governance suites are registered in the repo suite index and the SSOT contract files exist.

## Foundation hardening suite

Command `foundation-hardening` executes curated high-trust suites listed in `configs/suites/foundation_hardening.json`.

## SSOT rule

When a governance policy is changed, `bijux-dev-dag` command implementation and the owning contract documentation must be updated in the same change.

## SOURCE: docs/spec/DOCS_GOVERNANCE.md
# Documentation governance

## Allowed documentation taxonomy

All normative documentation must live under one of:

- `docs/spec/`
- `docs/architecture/`
- `docs/user/`
- `docs/dev/`
- `docs/reference/`
- `docs/tracking/`

`docs/generated/` is reserved for generated artifacts only.

## Root-doc budget

Root-level markdown files under `docs/` are capped at **100**.
Repository enforcement policy is defined in `configs/policy/docs_config_governance.json`.

## Required governance documents

- `docs/spec/WORKSPACE_CONTRACT.md`
- `docs/spec/BOUNDARY_RULES.md`
- `docs/spec/EVIDENCE_MODEL.md`
- `docs/spec/DOCS_GOVERNANCE.md`
- `docs/spec/MISSION_STATEMENT.md`
- `docs/spec/ROOT_MESSAGING_CONTRACT.md`
- `docs/tracking/DOC_OWNERSHIP.json`

## Templates

Contract docs must include:

- scope
- authority
- invariants
- allowed changes
- related tests
- related schemas

Architecture docs must include:

- purpose
- boundaries
- dependencies
- failure modes
- non-goals

User docs must include:

- audience
- prerequisites
- examples
- outputs
- failure behavior

## Content rules

- marketing maturity language is disallowed unless historically quoted
- unsupported guarantee language is disallowed without evidence links
- stale crate names and legacy paths are disallowed
- speculative roadmap content must live in `docs/tracking/`
- self-scoring scorecard documents are disallowed in root docs
- title overlap across root docs is rejected by governance tests

## Ownership

Normative docs require ownership metadata in `docs/tracking/DOC_OWNERSHIP.json`.

## SOURCE: docs/spec/REPOSITORY_STRUCTURAL_HEALTH_CONTRACT.md
# Repository Structural Health Contract

## Purpose

This contract defines durable repository-structure health guarantees for
`bijux-dag`. The goal is to keep module boundaries understandable, dependency
flow explicit, and structural regressions detectable.

## Required Structural Signals

- largest modules inventory
- highest churn module inventory
- lowest coverage module inventory
- duplicate helper detection
- unused module detection
- cyclic dependency detection
- repository dependency graph
- module ownership mapping
- module complexity scoring
- refactoring candidate inventory
- module documentation coverage report
- dependency drift verification
- hygiene regression fixtures
- structural health dashboard
- complexity benchmarks
- structural lint checks
- dependency verification checks
- architectural conformance checks
- repository health telemetry
- repository structure verification suite

## Determinism Rules

- Structural reports are reproducible for the same repository revision.
- Sorting and grouping rules must be stable.
- Dashboard summaries must not rely on nondeterministic file iteration order.

## Safety Rules

- Structural checks must not mutate runtime or evidence state.
- Failures are explicit and actionable.
- Contract tests anchor reports to concrete command and policy surfaces.


## SOURCE: docs/spec/ROOT_MESSAGING_CONTRACT.md
# Root Messaging Contract

## Scope

This contract governs root-level messaging surfaces (`README.md`, root docs, and release-facing summary text).

## Invariants

- Root one-liner must be exactly: `Git for computation graphs.`
- Root mission wording must align with `docs/spec/MISSION_STATEMENT.md`.
- Root docs must not imply execution support beyond `docs/reference/EXECUTION_SUPPORT_POLICY.md`.
- Experimental or simulated behavior must be explicitly labeled.
- Alternative drifting taglines are disallowed in root messaging.

## Oversell guardrails

The following claim patterns are disallowed in root messaging unless linked to conformance evidence:

- "full platform"
- "production-ready distributed orchestration"
- "drop-in replacement for Airflow"
- "drop-in replacement for Prefect"
- "drop-in replacement for Dagster"

## Related tests

- `crates/bijux-dev-dag/tests/root_messaging_contracts.rs`
- `crates/bijux-dev-dag/tests/release_evidence_linkage_contracts.rs`
