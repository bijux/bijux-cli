# Graph Identity Specification

Graph identity is the stable identifier of DAG semantics. It exists so replay planning, drift detection, and history navigation can compare *definitions* without mixing in run-attempt noise.

## Contract surface

This specification defines:
- identity-relevant DAG inputs,
- canonicalization and hashing requirements,
- invalid states,
- compatibility requirements when canonicalization policy changes.

This specification does not define CLI rendering or storage engine layout.

## Normative requirements

### Identity inputs

Graph identity MUST be computed from canonical semantic DAG content only. Identity-relevant fields are:
- node set,
- dependency edges,
- node execution semantics,
- semantic graph-level options declared identity-relevant by policy.

Identity-excluded fields are:
- comments,
- whitespace,
- non-semantic annotation fields,
- declaration order when order is not semantic.

### Derivation algorithm

A conforming implementation MUST:
1. parse the DAG definition,
2. produce a canonical semantic representation,
3. serialize canonical representation deterministically,
4. hash serialized bytes with an explicit identity policy version,
5. emit graph identity with policy version reference.

### Core rules

- `RULE-GID-001`: semantic-equivalent canonical DAG content MUST yield equal graph identity.
- `RULE-GID-002`: any semantic DAG change MUST yield different graph identity.
- `RULE-GID-003`: identity-excluded field changes MUST NOT change graph identity.
- `RULE-GID-004`: identity values MUST be interpreted with the policy version that produced them.

## Invalid states

- `INVALID-GID-CANONICALIZATION-FAILURE`: canonical semantic form cannot be produced.
- `INVALID-GID-UNKNOWN-POLICY`: identity policy version is missing or unsupported.
- `INVALID-GID-AMBIGUOUS-SEMANTICS`: parser cannot resolve semantic meaning deterministically.

Implementations MUST reject identity emission for invalid states.

## Identity-preserving and identity-changing edits

Identity-preserving edits:
- comment-only update,
- whitespace or formatting-only change,
- declaration reordering that does not alter semantic topology.

Identity-changing edits:
- add/remove dependency edge,
- modify node execution semantics,
- modify semantic graph option included in identity policy.

## Canonicalization policy evolution

When canonicalization rules change:
- policy version MUST change,
- cross-version comparisons MUST be treated as incompatible unless an explicit compatibility mapping is defined,
- tooling MUST report incompatibility explicitly, not as `equivalent`.

## Worked examples

Example: identity preserved by formatting change.

```text
Before: same nodes/edges, compact formatting
After : same nodes/edges, expanded formatting and comments
Result: same graph identity (g_44a...)
```

Example: identity changed by semantic edit.

```text
Before: node transform depends on validate
After : dependency removed
Result: different graph identity (g_44a... -> g_981...)
```

## Guarantees

- Graph identity is deterministic under a fixed canonicalization policy.
- Graph identity changes when semantic DAG content changes.
- Identity-excluded edits do not cause identity drift.

## Non-guarantees

- Equal graph identity does not guarantee equal runtime outcome in all environments.
- Graph identity does not encode run-attempt data.
- Graph identity does not prove artifact equivalence by itself.

## Next reading

- [DAG model contract](docs/06-specification/01-dag-model.md)
- [Run identity contract](docs/06-specification/05-run-identity.md)
- [Replay semantics contract](docs/06-specification/07-replay-semantics.md)
