# Run Model

Define the execution run lifecycle, state transitions, and run-level evidence contract.

Run behavior operationalizes DAG definitions into auditable execution outcomes.

## Explanation
Run entity fields:
- `run.id`: unique run identity.
- `run.graph_id`: identity of the DAG definition under execution.
- `run.started_at` / `run.finished_at`: temporal bounds.
- `run.status`: lifecycle state.
- `run.node_results`: per-node execution outcomes.
- `run.metadata`: optional annotations.

Canonical run lifecycle states:
- `planned`: run created, execution not started.
- `running`: one or more nodes are being executed.
- `succeeded`: all required nodes completed successfully.
- `failed`: terminal failure due to one or more node failures.
- `canceled`: execution terminated by explicit stop request.

State transition rules:
- `planned -> running` only once.
- terminal states are immutable (`succeeded`, `failed`, `canceled`).
- a run cannot transition between terminal states.

Formal run-state rules:
- RULE-RUN-001: each run MUST have one unique `run.id`.
- RULE-RUN-002: run MUST reference one `run.graph_id`.
- RULE-RUN-003: run state MUST follow legal transition graph.
- RULE-RUN-004: terminal state MUST be immutable once reached.
- RULE-RUN-005: node outcomes MUST be attributable to run identity.

Node result contract:
- each scheduled node yields exactly one terminal node outcome for that run attempt.
- node outcome includes status, timing, adapter/backend context, and output references.
- failed upstream dependencies can classify downstream nodes as blocked/skipped, depending on policy.

Run evidence contract:
- run directory materializes run-level metadata and node-level outcomes.
- run records must be sufficient for inspect, replay planning, and diff attribution.

Invalid state definitions:
- INVALID-RUN-MISSING-ID: run identity absent.
- INVALID-RUN-MISSING-GRAPH-REFERENCE: `run.graph_id` absent.
- INVALID-RUN-ILLEGAL-TRANSITION: state transition violates lifecycle graph.
- INVALID-RUN-TERMINAL-MUTATION: terminal run changed after completion.

Edge cases:
- canceled runs are valid terminal states with incomplete execution coverage.
- failed runs may still produce partial artifact sets and valid evidence records.
- repeated executions on same graph are valid with distinct run identities.

Compatibility notes:
- additional run metadata fields are compatible if core state/lifecycle semantics remain unchanged.
- lifecycle vocabulary expansion must preserve existing terminal-state meaning.

## Examples
```text
Run lifecycle example:
planned -> running -> failed

Reason:
- node "lint" succeeded
- node "test" failed with non-zero exit code
```

```text
Run record linkage:
run.id = r_9f1...
run.graph_id = g_44a...
node_result.test.artifacts = [a_12b..., a_712...]
```

## Guarantees
- Run lifecycle states and legal transitions are explicit and finite.
- Terminal run state is immutable once reached.
- Run records provide a stable evidence surface for inspection and comparison.

## Limitations
- This contract does not guarantee identical wall-clock timing across environments.
- Retry policy and orchestration policy may vary by implementation mode.
- Storage retention policy is operational and outside this model contract.

## Related
- `docs/06-specification/01-dag-model.md`
- `docs/06-specification/03-artifact-model.md`
- `docs/06-specification/05-run-identity.md`
- `docs/03-user-guide/04-run-history.md`

## Run classes and evidence interpretation

Run classes that MUST be distinguishable:

- successful run,
- failed run,
- canceled run,
- replayed run,
- imported run.

Replayed and imported runs are valid run records but have distinct provenance class and should not be silently merged with original native runs.

## Authoritative versus derived run data

Authoritative fields:

- run identity,
- graph identity reference,
- terminal status,
- node terminal outcomes,
- artifact linkage references.

Derived fields:

- summarized dashboards,
- trend aggregates,
- cached labels for convenience.

Contract rule: derived fields may be regenerated; authoritative fields define run truth.
