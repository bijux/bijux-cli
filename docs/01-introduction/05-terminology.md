# Terminology

## Purpose
Define canonical terms used across all documentation and remove ambiguity.

## Context
This glossary is the primary vocabulary source for user, architecture, specification, and operations documents.

## Explanation
Graph
- Canonical DAG definition object.

Pipeline
- Operational workflow view of a graph execution lifecycle.
- In bijux-dag, pipeline behavior is grounded in graph semantics.

Node
- Executable unit in a graph.

Run
- A concrete execution instance for a graph.

Attempt
- A single execution attempt within the lifecycle of a run.
- A run may contain multiple attempts under retry/recovery behavior.

Artifact
- Persisted output object from execution.

Output
- General produced data.
- In this docs set, "artifact" is the persistent tracked form of output.

Replay
- Validation-oriented re-execution/comparison process.

Diff
- Structured classification of change between comparable entities.

Inspect
- Introspection surfaces for execution and output state.

Determinism
- Stability of behavior under equivalent defined inputs/conditions.

Canonical distinction set:
- Artifact vs output:
  - Output is generic produced data.
  - Artifact is persisted, identity-tracked output.

- Run vs attempt:
  - Run is the full execution instance.
  - Attempt is one execution try inside a run lifecycle.

- Graph vs pipeline:
  - Graph is formal dependency definition.
  - Pipeline is operational execution framing built from that graph.

## Examples
```text
Example usage:
- "Run RUN_100 had two attempts."
- "Artifact ART_42 was produced by node transform_data."
- "Graph diff identified dependency edge changes."
```

## Guarantees
- Canonical term definitions here are normative for this docs tree.
- Distinction sets remove common ambiguity points.

## Limitations
- This glossary does not define schema fields or hashing algorithms.
- Contract-level semantics remain in specification docs.

## Related
- `docs/01-introduction/04-core-concepts.md`
- `docs/06-specification/04-graph-identity.md`
- `docs/06-specification/05-run-identity.md`
- `docs/06-specification/06-artifact-identity.md`
