# Artifacts

## Purpose
Explain artifact behavior, storage expectations, identity, and lineage.

## Context
Artifacts are the persisted outputs users inspect, compare, and transport.

## Explanation
Artifact basics:
- produced by node execution
- persisted for later inspection/replay/diff workflows
- tracked by artifact identity metadata

Artifact path guidance:
- write to explicit, deterministic paths
- avoid paths coupled to volatile runtime state
- keep path conventions consistent across nodes

Artifact hashing:
- hashing supports stable identification and comparison
- hash computation should be tied to artifact content contract

Artifact lineage:
- lineage links artifact back to run and producing node context
- lineage is used for traceability and debugging

## Examples
```bash
# Inspect artifact details
bijux-dag inspect artifact --artifact-id ART_20260309_001
```

```text
Artifact review checklist:
- artifact_id exists
- producing run_id exists
- producing node_id exists
- expected path exists
```

## Guarantees
- Artifact concepts here align with specification identity and artifact model documents.
- Path, hashing, and lineage are treated as first-class operational concerns.

## Limitations
- This guide does not define exact hashing algorithm internals.
- Storage backend implementation details are out of scope.

## Related
- `docs/03-user-guide/04-run-history.md`
- `docs/05-system-architecture/06-artifact-store.md`
- `docs/06-specification/03-artifact-model.md`
- `docs/06-specification/06-artifact-identity.md`
