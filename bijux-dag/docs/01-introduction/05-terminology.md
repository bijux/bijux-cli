# Terminology

This is the canonical vocabulary for bijux-dag docs. Use these meanings consistently.

## Canonical glossary

- `graph`: DAG definition state (nodes, dependencies, semantic config).
- `node`: executable unit inside a graph.
- `run`: one concrete execution instance of one graph state.
- `attempt`: one execution try inside a run lifecycle where retry behavior exists.
- `artifact`: persisted, identity-tracked output unit with lineage.
- `output`: generic produced data; use when identity/lineage tracking is not the point.
- `replay`: validation-oriented re-execution against baseline context.
- `diff`: scoped comparison classification across graph, run, or artifact surfaces.
- `inspect`: read-only retrieval of run/artifact evidence.
- `pipeline`: operational view of running and comparing graph executions over time.

## Preferred terms and discouraged synonyms

- Prefer `graph`; discourage `pipeline file` when meaning DAG definition.
- Prefer `run`; discourage `execution record` as primary term.
- Prefer `artifact`; discourage `output file` when lineage/identity matters.
- Prefer `replay classification`; discourage “rerun looked fine.”

## Distinctions that prevent confusion

- Artifact vs output:
  - artifact is persisted and identity-tracked;
  - output is generic produced data.
- Run vs attempt:
  - run is evidence container and identity anchor;
  - attempt is one try within run lifecycle.
- Graph vs pipeline:
  - graph is formal definition;
  - pipeline is operational execution history of graphs.

## Scope boundary

This glossary defines words, not algorithms. Field rules and invalid states live in specification docs.

## Next reading

- Relationship model using these terms: [Core Concepts](../01-introduction/04-core-concepts.md)
- Identity contracts: [Graph Identity](../06-specification/04-graph-identity.md), [Run Identity](../06-specification/05-run-identity.md), [Artifact Identity](../06-specification/06-artifact-identity.md)
