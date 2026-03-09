# Artifacts

Explain artifact behavior, storage expectations, identity, and lineage.

Artifacts are the persisted outputs users inspect, compare, and transport.

## Explanation
Artifact basics:
- produced by node execution
- persisted for later inspection/replay/diff workflows
- tracked by artifact identity metadata

Artifact lifecycle:
1. node execution produces output payload.
2. runtime classifies payload as artifact candidate.
3. artifact bytes and metadata are persisted.
4. artifact identity/hash is computed and indexed.
5. artifact is linked to run/node lineage.
6. inspect/replay/diff workflows consume artifact record.

Artifact path guidance:
- write to explicit, deterministic paths
- avoid paths coupled to volatile runtime state
- keep path conventions consistent across nodes

Artifact hashing:
- hashing supports stable identification and comparison
- hash computation should be tied to artifact content contract
- identical canonical content should produce identical artifact identity under same identity policy version

Artifact lineage:
- lineage links artifact back to run and producing node context
- lineage is used for traceability and debugging

Storage structure (user-level view):
- artifact payload location (file/object path)
- artifact metadata record (size/type/hash)
- lineage links (`graph_id`, `run_id`, `node_id`, `artifact_id`)
- index entry for lookup and diff comparison

## Examples
```bash
# Inspect artifact details
bijux-dag inspect artifact --artifact-id ART_20260309_001
```

```text
Artifact hashing example:
artifact payload: out/result.txt
hash algorithm: sha256
artifact hash: sha256:2f67...
```

```text
Artifact storage structure example:
artifacts/
  ART_20260309_001/
    payload.bin
    metadata.json
```

```mermaid
graph LR
  A[Graph g_44a] --> B[Run r_101]
  B --> C[Node transform]
  C --> D[Artifact a_712]
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
- Includes lifecycle, hashing, storage, and lineage usage in one operator flow.

## Limitations
- This guide does not define exact hashing algorithm internals.
- Storage backend implementation details are out of scope.

## Related
- `docs/03-user-guide/04-run-history.md`
- `docs/05-system-architecture/06-artifact-store.md`
- `docs/06-specification/03-artifact-model.md`
- `docs/06-specification/06-artifact-identity.md`
