# Run Identity

## Purpose
Define run identity derivation, uniqueness boundaries, and attribution guarantees.

## Context
Run identity links execution evidence to one concrete attempt of graph execution.

## Explanation
Run identity definition:
- run identity uniquely identifies a concrete run attempt.
- run identity is distinct from graph identity; multiple runs may share one graph identity.

Run identity input domains:
- graph identity reference.
- run creation context (timestamp/nonce/sequence as applicable).
- execution mode and selected adapter context when declared identity-relevant.

Formal run-identity rules:
- RULE-RID-001: each run attempt MUST map to one unique run identity.
- RULE-RID-002: repeated attempts MUST NOT reuse run identity.
- RULE-RID-003: run identity generation MUST be deterministic with respect to declared inputs.
- RULE-RID-004: run identity MUST be linkable to graph identity and node outcomes.

Uniqueness contract:
- no two run attempts in the same identity namespace may share the same run identity.
- retries create distinct run identities even when graph identity is unchanged.

Attribution contract:
- run identity is the parent link for node outcomes and artifacts.
- inspect/replay/diff workflows consume run identity as primary execution key.

Stability boundaries:
- run identity generation must be deterministic with respect to its generation function and supplied inputs.
- uniqueness mechanisms may include monotonic sequence or collision-resistant random domains.

Invalid state definitions:
- INVALID-RID-DUPLICATE: same run identity assigned to distinct run attempts.
- INVALID-RID-MISSING-GRAPH-LINK: run identity exists but graph reference missing.
- INVALID-RID-NONDETERMINISTIC-GENERATION: same declared inputs produce divergent run identities.

Edge cases:
- retry after transient failure is valid and must allocate a new run identity.
- same graph, same environment, different invocation time remains distinct run identity by attempt.

## Examples
```text
Repeated execution over same graph:
graph_id = g_44a...
run_1 = r_100...
run_2 = r_101...
Result: same graph identity, different run identities
```

```text
Run-to-artifact linkage:
run.id: r_101...
artifact.id: a_712...
artifact.run_id: r_101...
```

## Guarantees
- Run identity is unique per run attempt within identity namespace.
- Run identity provides stable attribution anchor for run evidence.
- Distinct run attempts are distinguishable even with identical DAG definitions.

## Limitations
- Run identity does not encode full artifact content.
- Equality of run identity is not intended across independent identity namespaces.
- This document does not define storage key layout details.

## Related
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/04-graph-identity.md`
- `docs/06-specification/06-artifact-identity.md`
- `docs/03-user-guide/04-run-history.md`
