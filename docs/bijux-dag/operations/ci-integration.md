---
title: CI Integration
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# CI Integration

A useful `bijux-dag` CI lane must prove more than compilation or help output.
It should identify the binary, validate the exact graph, execute against
explicit inputs, retain the run directory, and verify that retained evidence.

This page covers product-level CI. Repository formatting, lint, dependency
policy, broad tests, and release validation remain maintainer concerns.

## Choose The Lane

| Context | Recommended lane | Claim |
| --- | --- | --- |
| change inside this repository | `make dag-demo` | one checked-in workflow validates, runs cold and warm, retains an artifact, replays, and verifies strictly |
| downstream repository using an installed binary | explicit commands below | the selected binary executed the consumer graph and produced a strictly verified retained run |
| command-surface smoke check | `bijux-dag version` and `bijux-dag commands` | binary identity and stable command discovery only |

Do not report the smoke check as workflow execution evidence.

## Repository Proof Lane

From the `bijux-core` root:

```bash
make dag-demo
```

The target writes its inputs, structured command results, run directories,
cache, replay, and final verification under `artifacts/dag-demo/`. It checks
the final JSON fields rather than relying only on command exit status.

Retain that directory as a CI artifact when the demonstration is part of a
review or release decision. The [First-Run Tutorial](first-run-tutorial.md)
lists every assertion made by the target and the limits of the resulting
claim.

## Downstream Product Lane

Use a fixed binary version and consumer-owned graph. The following shape avoids
ambient run locations and human-output parsing:

```bash
GRAPH_PATH="path/to/workflow.dag.json"
RUNS_ROOT="artifacts/bijux-dag-ci/runs"
CACHE_ROOT="artifacts/bijux-dag-ci/cache"
RUN_ID="ci-${CI_COMMIT_SHA:-local}"

mkdir -p artifacts/bijux-dag-ci
bijux-dag version > artifacts/bijux-dag-ci/version.txt
bijux-dag --json validate "${GRAPH_PATH}" \
  > artifacts/bijux-dag-ci/validate.json
bijux-dag --json run "${GRAPH_PATH}" \
  --out "${RUNS_ROOT}" \
  --run-id "${RUN_ID}" \
  --cache readwrite \
  --cache-dir "${CACHE_ROOT}" \
  > artifacts/bijux-dag-ci/run.json
bijux-dag --json verify "${RUNS_ROOT}/run-${RUN_ID}" --strict \
  > artifacts/bijux-dag-ci/verify.json
```

Add every required graph input explicitly with `--input`. Never depend on a
developer's default run root, cache root, working directory, or shell state.
When untrusted code can reach shell or container adapters, use an isolated CI
runner appropriate to that threat; the DAG runtime is not a sandbox.

## Required Assertions

CI should parse the JSON envelopes and reject the run unless:

- validation reports `ok = true`;
- execution reports a successful finalized run with the expected run id;
- the finalized run directory exists under the configured root;
- strict verification reports `ok = true`, `status = "ok"`, and
  `mode = "strict"`;
- workload-specific output assertions pass.

Strict verification proves retained structural and integrity contracts. It
does not prove that a report, model, or transformed dataset is scientifically
or operationally correct. Consumer CI owns those domain assertions.

## Failure And Retention

Use shell failure propagation so a nonzero `validate`, `run`, or `verify` exit
fails the job. Preserve the partial structured output and retained run
directory before cleanup.

| Failed boundary | Keep | Do not claim |
| --- | --- | --- |
| binary discovery | version and install logs | graph compatibility |
| validation | validation envelope and graph revision | execution |
| run | run envelope and retained run directory | complete workflow |
| strict verification | verification envelope and run directory | trustworthy retained evidence |
| domain assertion | verified run plus actual output | correct business or scientific result |

Do not overwrite a failed run with a retry. Give the retry a new run id so
causal evidence remains inspectable.

## Release-Lane Discipline

Stable operator CI should use commands visible in `bijux-dag --help`.
Experimental explicit-path routes require an acknowledged compatibility risk.
Simulated and internal namespaces are repository evidence surfaces, not
substitutes for product CI.

The [Release Boundary](../foundation/release-boundary.md), backed by
`contracts/foundation/dag_release_truth_table.v1.json`, governs those lanes.

## Related Guidance

- [First-Run Tutorial](first-run-tutorial.md)
- [Run Evidence Layout](../interfaces/run-evidence-layout.md)
- [Execution Security And Isolation](security-isolation-truth.md)
- [Failure Recovery](failure-recovery.md)
