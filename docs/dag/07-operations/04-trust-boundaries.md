# Trust Boundaries

Trust boundaries define where evidence can be trusted as-is and where re-verification is required before decisions are made.

## Boundaries that matter

Operationally critical boundaries are:
- DAG input boundary,
- bundle import boundary,
- backend execution boundary,
- artifact persistence boundary,
- imported-run provenance boundary,
- local environment boundary (host/container/runtime configuration).

Crossing any boundary changes what can be assumed without new evidence.

## Safe and unsafe assumptions

Safe assumptions:
- validated DAG input with explicit schema and dependency checks,
- replay/diff classifications produced under declared policy and capability envelope,
- artifact lineage linked to producing run and node.

Unsafe assumptions:
- imported bundle treated as trusted without verification,
- `unknown` classification interpreted as equivalent,
- backend portability interpreted as backend equivalence,
- imported run history treated as local baseline without ancestry verification.

## Re-verify checklist for boundary crossings

When a boundary is crossed, re-verify at least:
1. input integrity (hash/signature/provenance as available),
2. capability envelope (backend and environment constraints),
3. lineage integrity (`graph_id`, `run_id`, `node_id`, `artifact_id` links),
4. replay/diff classification against accepted baseline,
5. policy compatibility version used by identity/replay/diff tooling.

Promotion MUST stop if any re-verification scope remains unresolved.

## Replay and portability interpretation across boundaries

Replay interpretation rule:
- replay `equivalent` is valid only for the declared boundary context and capability envelope.

Portability interpretation rule:
- portable evidence means transferable with bounded guarantees,
- it does not imply semantic equivalence across all backends,
- downgrade or `incomplete` classifications must be preserved during boundary transitions.

## Practical boundary workflow

Example: external bundle into release pipeline.

```text
import bundle -> verify integrity/provenance -> inspect lineage completeness
-> replay required scopes -> diff against approved baseline
-> classify equivalent/drift/incomplete -> decide promote or block
```

## Guarantees

- Boundary crossings require explicit re-verification.
- Unsafe assumption classes are explicitly named.
- Replay/portability interpretation is bounded to context.

## Non-guarantees

- Boundaries cannot compensate for full host compromise.
- Missing provenance cannot be reconstructed by policy text alone.

## Next reading

- [Security model](03-security-model.md)
- [Replay semantics contract](../06-specification/07-replay-semantics.md)
- [Portability architecture](../05-system-architecture/10-portability.md)
