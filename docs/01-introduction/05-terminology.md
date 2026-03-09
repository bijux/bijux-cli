# Terminology

Define canonical terms used across all documentation and remove ambiguity.

This glossary is the primary vocabulary source for user, architecture, specification, and operations documents.

## Explanation
Use these definitions exactly in docs and reviews. If a term is not listed here, define it before using it as contract language.

Graph
- Canonical DAG definition object: nodes + dependency edges + semantic configuration.
- Use "graph" for definition state, not for execution outcomes.

Pipeline
- Operational view of executing a graph over time (run history, artifacts, comparisons).
- Use when describing workflow operations, not schema structure.

Node
- Executable unit inside a graph with defined dependencies and execution intent.

Run
- One concrete execution instance of one graph definition.

Attempt
- One execution try inside a run lifecycle when retry/recovery is enabled.
- Default mental model: one run can contain one or more attempts.

Artifact
- Persisted output object with identity and lineage links.
- Preferred term for output data that is tracked and comparable.

Output
- Generic produced data.
- Use only when persistence/identity tracking is irrelevant.

Replay
- Validation-oriented re-execution against baseline graph/run context.

Diff
- Structured comparison classification across graph, run, or artifact scope.

Inspect
- Read-only introspection of run state, node outcomes, and artifacts.

Determinism
- Stability of classified behavior under equivalent defined inputs and supported environment constraints.

Portability
- Ability to transfer workflow context and outputs across environments with explicit equivalence boundaries.

Canonical distinction set:
- Artifact vs output:
  - output = generic produced bytes/data
  - artifact = persisted, identity-tracked output unit

- Run vs attempt:
  - run = full execution instance and evidence container
  - attempt = one try within the run lifecycle

- Graph vs pipeline:
  - graph = formal dependency definition
  - pipeline = operational execution framing built from graph runs over time

- Replay vs diff:
  - replay = produce candidate evidence through re-execution
  - diff = classify divergence between baseline and candidate evidence

Ambiguity policy:
- Do not use "job", "step", or "result bundle" as aliases unless explicitly mapped to canonical terms.
- Prefer canonical terms in headings, examples, and CLI explanations.

Preferred vs discouraged synonyms:
- prefer `graph`; discourage `pipeline file` when referring to formal DAG definition.
- prefer `run`; discourage `execution record` as primary term.
- prefer `artifact`; discourage `output file` when identity and lineage are relevant.
- prefer `replay classification`; discourage `rerun looked fine` as evidence statement.

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
- Each canonical term has one preferred meaning and usage boundary.

## Limitations
- This glossary does not define schema fields or hashing algorithms.
- Contract-level semantics remain in specification docs.

## Related
- `docs/01-introduction/04-core-concepts.md`
- `docs/03-user-guide/03-artifacts.md`
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
- `docs/06-specification/04-graph-identity.md`
