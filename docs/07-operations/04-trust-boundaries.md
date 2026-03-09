# Trust Boundaries

Define where trust assumptions begin and end across CLI, runtime, adapters, and storage.

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

Trust-boundary recommendations:
- validate all external DAG or bundle inputs before runtime admission.
- keep boundary-crossing events observable in logs and evidence records.
- require operator approval for privileged cross-zone operations.
- treat unknown trust states as blocking until resolved.

Safe assumptions vs unsafe assumptions:
- safe: validated DAG input + stable capability descriptor + explicit replay/diff evidence.
- unsafe: imported bundle trusted without verification.
- unsafe: unknown classification interpreted as equivalent.
- unsafe: backend capability gaps ignored during portability decisions.

If you cross this boundary, re-verify:
- DAG source changed -> rerun validation and baseline replay.
- backend/adapter changed -> rerun replay and diff before release decisions.
- imported bundle from external source -> verify integrity and lineage before execution.

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
- Includes concrete boundary-operation recommendations.

## Limitations
- Trust boundaries cannot compensate for compromised host kernels or hypervisors.
- Cross-organization evidence trust may need additional signing and attestation controls.
- This document does not define legal/compliance policy text.

## Related
- `docs/07-operations/03-security-model.md`
- `docs/06-specification/05-run-identity.md`
- `docs/06-specification/06-artifact-identity.md`
- `docs/06-specification/08-diff-semantics.md`
