# Basic Troubleshooting

This guide is for first-week failures where you need fast diagnosis, not generic advice.

## Failure classes and what to do first

### Invalid DAG

Symptoms:

- run fails before execution starts,
- validation reports unknown dependency, duplicate node ID, or cycle.

Debug sequence:

1. `bijux-dag dag validate --dag <path>`
2. fix the first structural error only,
3. re-run validation before trying execution.

### Failed node during run

Symptoms:

- run has terminal `failed`,
- one node is first failing node.

Debug sequence:

1. `bijux-dag inspect run --run-id <id>`
2. identify first failed node and reason class,
3. inspect that node's expected inputs/outputs,
4. rerun only after cause is identified.

### Missing artifact

Symptoms:

- downstream node fails with missing input,
- expected artifact not present in artifact inspect output.

Debug sequence:

1. `bijux-dag inspect artifact --run-id <id>`
2. verify producer node outcome in run inspect,
3. verify output path expectation in graph definition,
4. compare against known-good run with diff.

### Replay drift

Symptoms:

- replay completes with `drift` or `incomplete`.

Debug sequence:

1. `bijux-dag replay --run-id <baseline>`
2. `bijux-dag diff run --left <baseline> --right <replay>`
3. classify drift scope (graph/run/artifact),
4. check environment/toolchain differences before blaming graph changes.

### Import mismatch

Symptoms:

- imported evidence cannot be replayed equivalently,
- lineage or identity comparisons are incomplete.

Debug sequence:

1. validate imported run/artifact identifiers,
2. verify bundle provenance and integrity,
3. replay imported baseline,
4. diff against local known-good run to classify compatibility gap.

## What to inspect first

Use this checklist before changing code or graph files:

- CLI command resolves and `--help` works,
- DAG validates,
- run has known terminal status,
- first failed node is identified,
- artifact lineage for failure path is visible,
- replay/diff classification is captured.

## Common wrong assumption

“If I rerun enough times, I will understand the failure.” Reruns without inspect/replay/diff evidence usually hide root causes instead of exposing them.

## Next reading

- Full first-run evidence flow: [Running A Pipeline](../02-getting-started/03-running-a-pipeline.md)
- Deeper debugging workflows: [Inspect And Debug](../03-user-guide/07-inspect-and-debug.md)
