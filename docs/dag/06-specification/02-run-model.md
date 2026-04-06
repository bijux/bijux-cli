# Run Model

Normative contract for run objects, state transitions, and evidence truth boundaries.

## Terms

- Run: one execution instance anchored by run identity.
- Run class: provenance category (original, replayed, imported).
- Node outcome: terminal status record for one node in one run context.

## Required fields

- `run_id`
- `graph_id` reference
- `status`
- run timing envelope
- node outcome records
- artifact linkage references (when produced)

## State model

Allowed lifecycle states:

- planned
- running
- succeeded
- failed
- canceled

Transition rules:

- RULE-RUN-001: run MUST have unique `run_id` in namespace.
- RULE-RUN-002: `planned -> running` occurs at most once.
- RULE-RUN-003: terminal states are immutable.
- RULE-RUN-004: run MUST retain graph reference for attribution.

## Run classes

Run records MUST distinguish:

- original run,
- replayed run,
- imported run.

Failed/canceled are status classes, not provenance classes.

## Authoritative versus derived data

Authoritative data:

- run identity,
- graph linkage,
- terminal status,
- node terminal outcomes,
- artifact references.

Derived data:

- summaries,
- dashboards,
- cached trend artifacts.

RULE-RUN-005: derived data MUST NOT override authoritative run facts.

## Invalid states

- INVALID-RUN-MISSING-ID
- INVALID-RUN-MISSING-GRAPH-LINK
- INVALID-RUN-ILLEGAL-TRANSITION
- INVALID-RUN-TERMINAL-MUTATION

## Next reading

- Execution identity relation: [Run Identity](../06-specification/05-run-identity.md)
- Output linkage contract: [Artifact Model](../06-specification/03-artifact-model.md)
