# Example Command Catalog

This catalog maps common DAG/operator commands to deterministic example assets and
their primary command entrypoint.

| command id | example graph | primary command |
| --- | --- | --- |
| `validate` | `evidence/dag/authoring/examples/hello.dag.json` | `bijux-dag validate --json evidence/dag/authoring/examples/hello.dag.json` |
| `plan` | `evidence/dag/authoring/examples/hello.dag.json` | `bijux-dag plan explain --json evidence/dag/authoring/examples/hello.dag.json` |
| `run` | `evidence/dag/authoring/examples/etl-constant-to-shell.dag.json` | `bijux-dag run --json evidence/dag/authoring/examples/etl-constant-to-shell.dag.json --out ${RUN_ROOT}` |
| `replay` | `evidence/dag/authoring/examples/replay-heavy-branching.dag.json` | `bijux-dag replay --json ${RUN_DIR} --out ${REPLAY_ROOT}` |
| `diff` | `evidence/dag/authoring/examples/replay-heavy-branching.dag.json` | `bijux-dag diff --json ${RUN_DIR_A} ${RUN_DIR_B}` |
| `cache` | `evidence/dag/authoring/examples/cached-branched-report.dag.json` | `bijux-dag run --json evidence/dag/authoring/examples/cached-branched-report.dag.json --cache-mode read-write --out ${RUN_ROOT}` |
| `artifact` | `evidence/dag/authoring/examples/multi-output-artifact.dag.json` | `bijux-dag artifact inspect --json ${RUN_DIR} --id ${ARTIFACT_ID}` |
| `app-mount` | `evidence/dag/authoring/examples/minimal_consumer.dag.json` | `bijux apps list --json` |
| `plugin` | `evidence/dag/authoring/examples/minimal_consumer.dag.json` | `bijux plugins list --json` |
| `bundle` | `evidence/dag/authoring/examples/replay-heavy-branching.dag.json` | `bijux-dag runs diagnostics-bundle ${RUN_ID} --root ${RUN_ROOT} --out ${BUNDLE_PATH} --json --redact` |
