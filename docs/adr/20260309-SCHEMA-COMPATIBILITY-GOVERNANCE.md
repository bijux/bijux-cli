# Schema Compatibility Governance

Status: accepted
Owner: schema maintainers
Date: 2026-03-09

## Decision
Schema evolution follows explicit compatibility policy with governed deprecation windows and verification gates.

## Consequences
- Versioned schema contracts are authoritative.
- Incompatible schema changes require planned migration path.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-SCHEMA-COMPATIBILITY-GUARANTEES.md
# ADR: Schema Compatibility Guarantees

## Status

Accepted

## Context

Execution portability and operator trust depend on deterministic schema compatibility rules that are enforced by tests and CI controls, not by convention.

## Decision

We guarantee the following:

1. Stable schema files are hash-frozen and drift is merge-blocking until compatibility review is explicit.
2. Supported historical versions remain accepted for declared compatibility windows.
3. Unsupported versions are rejected with classified failures.
4. Compatibility fixtures are mandatory for graph, run, artifact, proof, diff, and explain surfaces.
5. Schema policy documentation and changelog artifacts are mandatory governance outputs.

## Consequences

- Schema changes require explicit migration and changelog work.
- CI failures surface compatibility drift early.
- Operators receive durable compatibility diagnostics through generated reports and dashboards.

## Enforcement

- `crates/bijux-dev-dag/tests/schema_governance_contracts.rs`
- `crates/bijux-dev-dag/tests/schema_evolution_completion_contracts.rs`
- `crates/bijux-dev-dag/tests/proof_schema_compatibility_contracts.rs`
- `crates/bijux-dev-dag/tests/schema_compatibility_guarantees_contracts.rs`

### SOURCE: 20260308-AUTHORITATIVE-SCHEMA-RESIDENCY.md
# ADR: Authoritative Schema Residency

## Status
Accepted

## Context
Schema and output contract definitions were at risk of duplication across runtime/app/dev governance layers.

## Decision
Authoritative schema ownership is fixed as:
- DAG semantic schema and canonicalization contracts: `bijux-dag-core`.
- Runtime manifest/event/trace execution contracts: `bijux-dag-runtime`.
- Artifact identity, lineage, and persistence contracts: `bijux-dag-artifacts`.
- App JSON envelopes and command-level presentation contracts: `bijux-dag-app`.

`bijux-dev-dag` may validate and report against these schemas but must not become the source of truth.

## Consequences
- Release reviews have a single authority per schema family.
- Compatibility checks can target stable ownership locations.
- Duplicate schema drift is reduced.

### SOURCE: 20260308-OUTPUT-SCHEMA-GOVERNANCE-END-STATE.md
# ADR: Output and Schema Governance End-State

- Date: 2026-03-08
- Status: Accepted

## Context

Stable JSON command outputs must remain contract-safe for operators and automation. Existing schema references, command mappings, and lockstep tests were distributed and hard to audit for completeness.

## Decision

Adopt one governed source for stable JSON output coverage:

- Policy: `configs/policy/json_output_governance.json`
- Generated evidence:
  - command-to-schema inventory
  - schema-to-command-and-lockstep inventory
  - missing example report
  - missing lockstep report
  - schema registry page
  - stable JSON command registry page
- Required artifacts per schema:
  - minimal example output
  - maximal example output
  - lockstep test marker

## Consequences

Positive:

- Missing JSON contract artifacts are visible and release-gated.
- Schema and output ownership remain explicit and audit-friendly.
- Freshness checks are deterministic and easy to regenerate.

Tradeoff:

- Governance policy and generated docs must be refreshed when stable JSON surfaces change.

## Follow-up

- Keep `cargo run -p bijux-dev-dag --bin generate_json_output_governance_reports` in release workflows.
- Treat non-zero gap report counts as blocking until resolved.
