# POLICY AND COMPLIANCE GOVERNANCE

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ANTI_DRIFT_POLICY.md
# Anti-Drift Policy

## Drift classes
- docs drift
- schema drift
- contract drift
- cli drift
- test drift
- fixture drift
- benchmark drift
- dependency drift

## Drift blocker severities
- blocker: fails governance checks
- warning: reported but does not fail governance checks

## Required same-change alignment rule
Any change to a normative surface must update:
- owning contract doc
- tests or fixtures proving behavior
- user-facing docs that describe behavior

## Required checks
- command docs align with command tree
- JSON output docs align with schemas
- invariants docs align with invariant registry
- contract references align with tests
- docs examples align with executable fixtures
- crate graph docs align with cargo metadata evidence
- version support docs align with compatibility fixtures
- benchmark docs align with scenario definitions
- release policy docs align with release-readiness verification suite

## Repository trust summary
`bijux-dev-dag repo-trust-summary` is the control-plane command for domain trust status.

## SOURCE: docs/spec/AS_UNDERSCORE_IMPORT_POLICY.md
# As-Underscore Import Policy

## Purpose

`use ... as _;` is allowed only where it is the clearest way to satisfy dependency-touch or trait-visibility boundaries.

## Allowed cases

- Crate integration tests under `crates/*/tests/*.rs` for dependency-touch imports required by strict `unused-crate-dependencies` hygiene.
- Bench targets under `crates/*/benches/*.rs` for dependency-touch imports.
- Crate root entrypoints (`src/lib.rs`, `src/main.rs`, `src/bin/*.rs`) when dependency-touch imports are required by target-level dependency accounting.

## Disallowed cases

- Internal module implementation files outside test/bench and crate root entrypoints.
- Decorative `as _` aliases where a normal explicit import is clearer.

## Enforcement

- Policy source: `configs/policy/as_underscore_import_policy.json`
- Contract test: `crates/bijux-dev-dag/tests/as_underscore_import_contracts.rs`
- Audit report: `docs/reports/foundation/AS_UNDERSCORE_IMPORT_AUDIT.md`

## Review rule

Every new `use ... as _;` must fit one allowed path class or an explicit exception with a written reason.

## SOURCE: docs/spec/BOUNDARY_RULES.md
# Boundary rules

## Forbidden crate dependencies

The following dependencies are forbidden:

- `bijux-dag-runtime -> bijux-dag-app`
- `bijux-dag-runtime -> bijux-dag-cli`
- `bijux-dag-core -> bijux-dag-runtime`
- `bijux-dev-dag -> bijux-dag-runtime`

Additional policy boundaries:

- `bijux-dag-cli` must not depend directly on runtime internals (`bijux-dag-runtime`) or core semantics (`bijux-dag-core`).
- No crate may depend on another crate only to reuse formatting or JSON rendering helpers.

## Rationale

- Prevent execution internals from leaking into app and CLI orchestration layers.
- Keep core independent of runtime policy and side-effect boundaries.
- Keep development control-plane tooling independent from runtime internals.
- Keep the binary CLI as wiring-only.
- Keep display and rendering helpers local to their owning crate unless promoted to a neutral utility crate.

## Source of truth

Machine-enforced rules live in `configs/policy/dependency_rules.json`.

## SOURCE: docs/spec/FEATURE_DEVELOPMENT_FREEZE_POLICY.md
# Feature development freeze policy

## Rule
No new feature surfaces are introduced until foundation readiness criteria are satisfied.

## Evidence governance linkage
All new scenario-like assets must comply with `evidence/CONTRACT.md` and be registered in `evidence/ownership/evidence_ledger.json`.
Repository proof pillars are frozen: no new top-level proof roots beyond `evidence/`.

## Allowed during freeze
- contract clarification
- governance enforcement
- correctness fixes
- migration and compatibility safety work

## Disallowed during freeze
- new product surfaces without matching foundation evidence
- speculative runtime expansion without ownership and contract mapping
- scenario-like files added outside evidence-governed roots

## Lift condition
Freeze is lifted only when the foundation final report confirms readiness criteria satisfaction.

## SOURCE: docs/spec/INTERNAL_CONTRACT_DISCIPLINE_POLICY.md
# Internal Contract Discipline Policy

## Objective

Keep internal contracts explicit, directly tested, owned, and linked to fixtures/docs.

## Rules

1. Every internal contract must have a direct test and explicit owner.
2. Stable internal contracts must include docs/spec linkage.
3. Contract-to-fixture and contract-to-suite mappings must be generated and current.
4. Contract drift detection must run in governance suites.

## Enforcement

- `configs/policy/internal_contract_governance.json`
- `configs/suites/internal_contract_verification.json`
- `crates/bijux-dev-dag/tests/internal_contract_governance_contracts.rs`

## SOURCE: docs/spec/MIGRATION_POLICY.md
# Migration Policy

## Supported migration modes
- Automatic: no-op format-preserving migrations (`from == to`).
- Manual: operator-managed rewrite with explicit report output.
- Unsupported: cross-major format jumps.

## Current support boundary
This repository currently supports no-op migration assertions and explicit rejection for unsupported migrations.
No broad automatic schema/run/export migration is claimed.

## Migration report format
Migration reports must include:
- source version
- target version
- changed fields
- dropped/unrepresentable fields
- status (`no-op` / `applied` / `rejected`)

## SOURCE: docs/spec/PLACEHOLDER_SURFACE_POLICY.md
# Placeholder Surface Policy

## Purpose

Prevent fake completeness by banning placeholder implementations in stable code paths and evidence roots.

## Rules

- Stable source code must not contain `todo!(`, `unimplemented!(`, or panic text that says implementation is missing.
- Public-facing command/output surfaces must not contain placeholder wording unless explicitly allowlisted with an owner and deadline.
- Battle scenarios, release-blocking evidence assets, and operator-facing stable command paths must remain placeholder-free.

## Controlled exceptions

- Object-store runtime boundary message remains explicit until an approved backend contract is implemented.

## Governance artifacts

- Policy: `configs/policy/placeholder_surface_policy.json`
- Inventory report: `docs/reports/foundation/PLACEHOLDER_INVENTORY_REPORT.md`
- Removal report: `docs/reports/foundation/placeholder_removal_report.md`
- Retention report: `docs/reports/foundation/placeholder_retention_report.md`
- Enforcement test: `crates/bijux-dev-dag/tests/placeholder_surface_contracts.rs`

## SOURCE: docs/spec/POLICY_CONTRACT.md
# Policy Contract

## Scope
Defines policy inputs, enforcement points, and decision visibility.

## Invariants
- Policy evaluation is deterministic for identical inputs.
- Deny decisions include a machine-readable reason.
- Debug mode may emit policy evaluation traces.
- Policy traces follow `docs/spec/POLICY_EVALUATION_TRACE.md`.

## Related tests
- `evidence/battle/workflows/policy/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Related schemas
- `configs/schema/dag.schema.json`

## Versioning and change policy
Policy input changes require schema and docs updates in the same change.

## SOURCE: docs/spec/POLICY_EVALUATION_TRACE.md
# Policy Evaluation Trace

## Scope
Defines debug-mode policy trace expectations.

## Contract
- Debug output includes policy rule decisions (`allow`/`deny`) with rule labels.
- Default output excludes detailed policy trace internals.
- Trace format is machine-readable JSON.

## Related tests
- `crates/bijux-dag-app/tests/policy_mode_contract.rs`

## Versioning and change policy
Trace field additions are additive. Field removals require contract update and snapshot refresh.

## SOURCE: docs/spec/RELEASE_POLICY.md
# Release Policy

## Scope
Defines the minimum evidence required before a release is allowed.

## Release gate requirements
- Contract coverage report has no missing entries.
- Schema coverage report has no missing positive/negative fixture families.
- Docs coverage and taxonomy checks pass.
- Test suites (`checks`, `tests`, `contracts`, `repo`, `docs`) pass.
- E2E matrix report is available and passing.
- Benchmark comparison against previous baseline is within thresholds.
- Resource profile comparison against previous baseline is within accepted thresholds.
- Compatibility matrix for supported schema/graph versions is generated.
- Known limitations are explicitly documented for the release.

## Release blocker classes
- missing contracts
- missing schemas
- missing e2e evidence
- unreviewed performance regression
- undocumented breaking change

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs`
- `bijux-dev-dag release post-release-verify`

## Versioning and change policy
Any relaxation of release requirements is a breaking governance change and requires explicit changelog entry.

## SOURCE: docs/spec/RUNTIME_OVERREACH_REDUCTION_POLICY.md
# Runtime Overreach Reduction Policy

## Goal

Keep `bijux-dag-runtime` focused on deterministic execution, planning, state-machine legality, artifact integrity, and trust-critical boundaries.

## Rule

Runtime modules classified as `move` in `configs/policy/runtime_overreach_cleanup.json` must not become release-evidence requirements and must not expand runtime kernel authority.

## Enforcement

- Contract test: `crates/bijux-dev-dag/tests/runtime_overreach_contracts.rs`
- Report: `docs/reports/foundation/RUNTIME_OVERREACH_BEFORE_AFTER_REPORT.md`

## Scope decisions

- Keep: semantic lineage storage integrity required by artifact trust boundaries.
- Move: AI assist, workflow productization, ecosystem adoption/packaging, adaptive/cost models, federated/geo/HA scheduler surfaces, control-plane API in runtime, provenance compliance policy.

## SOURCE: docs/spec/RUNTIME_SCOPE_GOVERNANCE_POLICY.md
# Runtime Scope Governance Policy

## Lifecycle classes

Every runtime module must declare one lifecycle class:

- `core`
- `adapter`
- `operator-support`
- `experimental`
- `speculative`

## Required controls

1. New runtime modules require explicit lifecycle declaration before merge.
2. `experimental` and `speculative` runtime modules require explicit expiration criteria.
3. `experimental` and `speculative` runtime modules must remain quarantined from default operator surfaces.
4. Quarantined modules must not be presented as stable capability guarantees.

## Enforcement

- `configs/policy/runtime_module_lifecycle_status.json`
- `crates/bijux-dev-dag/tests/runtime_scope_contraction_guarantees_contracts.rs`
- `configs/suites/runtime_scope_contraction_verification.json`

## SOURCE: docs/spec/TRUTH_BEFORE_CONVENIENCE_DOCTRINE.md
# Truth Before Convenience Doctrine

## Rule

Bijux DAG prioritizes truthful behavior and evidence-backed claims over convenience shortcuts.

## Invariants

- If a convenience behavior hides semantic uncertainty, truth wins.
- If a fast path can misrepresent trust state, it must be rejected or labeled.
- If messaging conflicts with implemented evidence, messaging must be corrected.

## Enforcement

- Release evidence reports are mandatory.
- Blocking vs advisory evidence must remain explicit.
- Drift checks must fail on ambiguous release classification.

## SOURCE: docs/spec/appendices/runtime/RUNTIME_SCOPE_GOVERNANCE_POLICY.md
# Runtime Scope Governance Policy

## Lifecycle classes

Every runtime module must declare one lifecycle class:

- `core`
- `adapter`
- `operator-support`
- `experimental`
- `speculative`

## Required controls

1. New runtime modules require explicit lifecycle declaration before merge.
2. `experimental` and `speculative` runtime modules require explicit expiration criteria.
3. `experimental` and `speculative` runtime modules must remain quarantined from default operator surfaces.
4. Quarantined modules must not be presented as stable capability guarantees.

## Enforcement

- `configs/policy/runtime_module_lifecycle_status.json`
- `crates/bijux-dev-dag/tests/runtime_scope_contraction_guarantees_contracts.rs`
- `configs/suites/runtime_scope_contraction_verification.json`
