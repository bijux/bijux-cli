# Core Concepts

## Purpose
Provide a practical conceptual map for operating and understanding bijux-dag.

## Context
This document establishes the primary objects and relationships used throughout user guides, CLI reference, and specification docs.

## Explanation
DAG
- A directed acyclic graph that defines nodes and dependency edges.
- It describes allowed execution order without cycles.

Node
- A unit of execution with declared inputs, behavior, and outputs.
- Nodes are scheduled when dependency prerequisites are satisfied.

Dependency edge
- A directional relation from prerequisite node to dependent node.
- Encodes execution ordering constraints.

Run
- A concrete execution instance of a DAG under a specific input and environment context.
- Produces outcomes, traces, and artifacts.

Artifact
- A persisted output object produced by workflow execution.
- Artifacts carry identity/provenance-relevant metadata.

Identity model
- Graph identity: identifies workflow definition state.
- Run identity: identifies execution instance.
- Artifact identity: identifies produced output objects.

Replay
- Re-execution and validation process to compare behavior across runs or contexts.
- Used for confidence and diagnosis.

Diff
- Structured comparison between two comparable entities (graph, run, artifact).
- Used to classify change and isolate cause.

Inspect
- Operational introspection for run state, artifacts, and diagnostics.

Concept boundaries:
- Graph is definition; run is execution.
- Artifact is produced output; run is execution context.
- Replay validates behavior over time; diff classifies what changed.

## Examples
```text
Typical workflow concept flow:
Graph definition -> Run execution -> Artifact production -> Inspect diagnostics -> Replay validation -> Diff analysis
```

## Guarantees
- Core concepts are defined once and reused consistently.
- Concept boundaries here are aligned with specification section structure.

## Limitations
- This document is conceptual, not a contract specification.
- Field-level schemas and algorithms are defined in specification docs.

## Related
- `docs/01-introduction/05-terminology.md`
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/03-artifact-model.md`
- `docs/06-specification/07-replay-semantics.md`
