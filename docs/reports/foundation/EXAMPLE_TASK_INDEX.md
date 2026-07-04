# Example Task Index

This index maps common DAG/operator tasks to deterministic example assets and
their primary command entrypoint.

| task | example graph | primary command |
| --- | --- | --- |
| `validate` | `evidence/authoring/examples/hello.dag.json` | `bijux-dag validate --json evidence/authoring/examples/hello.dag.json` |
| `plan` | `evidence/authoring/examples/hello.dag.json` | `bijux-dag plan explain --json evidence/authoring/examples/hello.dag.json` |
| `run` | `evidence/authoring/examples/etl-constant-to-shell.dag.json` | `bijux-dag run --json evidence/authoring/examples/etl-constant-to-shell.dag.json --out ${RUN_ROOT}` |
| `replay` | `evidence/authoring/examples/replay-heavy-branching.dag.json` | `bijux-dag replay --json ${RUN_DIR} --out ${REPLAY_ROOT}` |
| `diff` | `evidence/authoring/examples/replay-heavy-branching.dag.json` | `bijux-dag diff --json ${RUN_DIR_A} ${RUN_DIR_B}` |
| `cache` | `evidence/authoring/examples/cached-branched-report.dag.json` | `bijux-dag run --json evidence/authoring/examples/cached-branched-report.dag.json --cache-mode read-write --out ${RUN_ROOT}` |
| `artifact` | `evidence/authoring/examples/multi-output-artifact.dag.json` | `bijux-dag artifact inspect --json ${RUN_DIR} --id ${ARTIFACT_ID}` |
| `app-mount` | `evidence/authoring/examples/minimal_consumer.dag.json` | `bijux apps list --json` |
| `plugin` | `evidence/authoring/examples/minimal_consumer.dag.json` | `bijux plugins list --json` |
| `bundle` | `evidence/authoring/examples/replay-heavy-branching.dag.json` | `bijux-dag runs diagnostics-bundle ${RUN_ID} --root ${RUN_ROOT} --out ${BUNDLE_PATH} --json --redact` |
