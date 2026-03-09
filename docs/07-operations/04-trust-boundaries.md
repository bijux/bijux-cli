# Trust Boundaries

## Purpose
Define where trust assumptions begin and end across CLI, runtime, adapters, and storage.

## Context
Trust boundaries govern how evidence and execution claims are interpreted and enforced.

## Explanation
Primary trust zones:
- authoring zone: DAG files and configuration sources.
- control zone: CLI and orchestration surfaces.
- execution zone: runtime and adapter-managed command execution.
- evidence zone: run directory and artifact persistence.

Boundary rules:
- untrusted authoring input must be validated before execution.
- adapter output is accepted only through normalized runtime contracts.
- evidence writes must preserve attribution (`graph_id`, `run_id`, `node_id`, `artifact_id`).
- trust does not transitively extend from one zone to another without explicit validation.

Operational implications:
- replay/diff provides verification across trust zone transitions.
- unknown states must remain explicit; never coerce unknown to equivalent.
- privileged operations require explicit and auditable controls.

## Examples
```text
Boundary crossing example:
untrusted DAG file -> schema and dependency validation -> schedulable plan -> execution
```

```text
Evidence trust chain:
node result -> run record -> artifact lineage -> replay/diff comparison
```

## Guarantees
- Trust zones and crossing rules are explicit and auditable.
- Lineage-aware evidence boundaries are preserved in operational model.
- Verification workflows are required when moving evidence across zones.

## Limitations
- Trust boundaries cannot compensate for compromised host kernels or hypervisors.
- Cross-organization evidence trust may need additional signing and attestation controls.
- This document does not define legal/compliance policy text.

## Related
- `docs/07-operations/03-security-model.md`
- `docs/06-specification/05-run-identity.md`
- `docs/06-specification/06-artifact-identity.md`
- `docs/06-specification/08-diff-semantics.md`
