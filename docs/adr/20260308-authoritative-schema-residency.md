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
