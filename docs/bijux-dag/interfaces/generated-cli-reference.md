---
title: Generated CLI Reference
audience: operators
type: generated-reference
status: canonical
owner: bijux-dag-docs
generated_from: bijux-dag clap help surface
---

# Generated CLI Reference

This page is generated from the live `bijux-dag` Clap command definitions.
It records the stable `v0.4.0` operator surface exactly as the product presents
it through `bijux-dag --help`. Experimental, simulated, and internal routes are
deliberately excluded from this page and listed separately in
[`reference/gated-command-inventory.md`](reference/gated-command-inventory.md).

## Placeholder Conventions

- `${GRAPH}`: DAG graph file such as
  `evidence/dag/authoring/examples/file-processing-report.dag.json`
- `${GRAPH_A}` and `${GRAPH_B}`: two graph revisions to compare
- `${RUNS_ROOT}`: retained run root such as `artifacts/bijux-dag/runs`
- `${RUN_DIR}`: one retained run directory such as `${RUNS_ROOT}/run-20260708-101500`
- `${RUN_DIR_A}` and `${RUN_DIR_B}`: two retained run directories
- `${RUN_ID}`, `${RUN_ID_A}`, `${RUN_ID_B}`: retained run ids under `${RUNS_ROOT}`
- `${CACHE_DIR}`: cache root such as `artifacts/bijux-dag/cache`
- `${NODE_FINGERPRINT}`: one persisted node fingerprint for cache packing
- `${CACHE_KEY}`, `${CACHE_KEY_A}`, `${CACHE_KEY_B}`: cache keys reported by `cache explain` or `run` output
- `${ARTIFACT_ID}`: artifact id such as `report` or `summary`
- `${DELIVERABLES_ROOT}`: deliverables root such as `artifacts/bijux-dag/deliverables`
- `${DIAGNOSTICS_DIR}`: diagnostics bundle output directory such as `artifacts/bijux-dag/run-diagnostics`

## Stable Root Surface

- `validate`
- `artifact-inspect`
- `artifact`
- `commands`
- `plan`
- `run`
- `replay`
- `runs`
- `diff`
- `explain`
- `verify`
- `doctor`
- `cache`
- `version`

## Root Help

```text
bijux-dag v0.4.0 is a local-first DAG runtime for reproducible workflows with explicit graph contracts, deterministic execution records, verified artifacts, cache explanation, and replayable run bundles.

Usage: bijux-dag [OPTIONS] [COMMAND]

Commands:
  validate
  artifact-inspect
  artifact
  commands
  plan
  run
  replay
  runs
  diff
  explain
  verify
  doctor
  cache
  version
  help              Print this message or the help of the given subcommand(s)

Options:
      --json

      --quiet

  -h, --help
          Print help

v0.4.0 surface truth table:
  stable: validate, plan, run, replay, runs ..., artifact, artifact-inspect, diff, explain, verify, doctor, cache, version, commands
  experimental: hidden explicit-path routes require deliberate inventory with `bijux-dag commands --lane experimental`
  simulated: modeled platform namespaces require `bijux-dag commands --lane simulated` to inventory and BIJUX_DAG_ENABLE_SIMULATED=1 to execute
  internal: maintainer namespaces require `bijux-dag commands --lane internal` to inventory and BIJUX_DAG_ENABLE_INTERNAL=1 to execute
  future: generic hpc beyond the shared-filesystem slurm lane, public remote workers, and public scheduler services are not part of v0.4.0

Use `bijux-dag commands` for the stable operator surface and add `--lane` only when you intentionally need repository-owned non-stable routes.
```

## `validate`

### Examples

- Validate one graph before planning or execution:

```bash
bijux-dag validate ${GRAPH}
```

- Emit validation diagnostics in JSON:

```bash
bijux-dag --json validate ${GRAPH}
```

### Help

```text
Usage: validate [OPTIONS] <DAGS>...

Arguments:
  <DAGS>...

Options:
      --strict

      --print-fingerprints

      --explain

  -h, --help
          Print help
```

## `artifact-inspect`

### Examples

- Inspect one retained artifact by run directory and artifact id:

```bash
bijux-dag artifact-inspect ${RUN_DIR} ${ARTIFACT_ID}
```

- Emit artifact inspection in JSON:

```bash
bijux-dag --json artifact-inspect ${RUN_DIR} ${ARTIFACT_ID}
```

### Help

```text
Usage: artifact-inspect <RUN_DIR> <ARTIFACT_ID>

Arguments:
  <RUN_DIR>

  <ARTIFACT_ID>

Options:
  -h, --help
          Print help
```

## `artifact`

### Examples

- List retained artifacts for one run:

```bash
bijux-dag artifact registry ${RUN_DIR}
```

- Emit machine-readable lineage for one retained artifact:

```bash
bijux-dag --json artifact lineage ${RUN_DIR} --artifact-id ${ARTIFACT_ID}
```

### Help

```text
Usage: artifact <COMMAND>

Commands:
  registry
  lineage
  promote
  retention
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `artifact registry`

#### Examples

- Inspect the retained artifact registry for one run:

```bash
bijux-dag artifact registry ${RUN_DIR}
```

- Inspect the same registry in JSON:

```bash
bijux-dag --json artifact registry ${RUN_DIR}
```

#### Help

```text
Usage: registry <RUN_DIR>

Arguments:
  <RUN_DIR>

Options:
  -h, --help
          Print help
```

### `artifact lineage`

#### Examples

- Show lineage for every retained artifact in one run:

```bash
bijux-dag artifact lineage ${RUN_DIR}
```

- Focus lineage output on one artifact id in JSON:

```bash
bijux-dag --json artifact lineage ${RUN_DIR} --artifact-id ${ARTIFACT_ID}
```

#### Help

```text
Usage: lineage [OPTIONS] <RUN_DIR>

Arguments:
  <RUN_DIR>

Options:
      --artifact-id <ARTIFACT_ID>

  -h, --help
          Print help
```

### `artifact promote`

#### Examples

- Promote one retained artifact into the release deliverables tree:

```bash
bijux-dag artifact promote ${RUN_DIR} ${ARTIFACT_ID} --deliverables-root ${DELIVERABLES_ROOT}
```

- Emit the promotion result in JSON for automation:

```bash
bijux-dag --json artifact promote ${RUN_DIR} ${ARTIFACT_ID} --deliverables-root ${DELIVERABLES_ROOT} --to release
```

#### Help

```text
Usage: promote [OPTIONS] --deliverables-root <DELIVERABLES_ROOT> <RUN_DIR> <ARTIFACT_ID>

Arguments:
  <RUN_DIR>

  <ARTIFACT_ID>

Options:
      --deliverables-root <DELIVERABLES_ROOT>

      --to <TO>
          [default: release]

  -h, --help
          Print help
```

### `artifact retention`

#### Examples

- Summarize retained artifact roots under one runs directory:

```bash
bijux-dag artifact retention ${RUNS_ROOT}
```

- Inspect retained artifact roots in JSON:

```bash
bijux-dag --json artifact retention ${RUNS_ROOT}
```

#### Help

```text
Usage: retention <ROOT>

Arguments:
  <ROOT>

Options:
  -h, --help
          Print help
```

## `commands`

### Examples

- List the stable public operator surface:

```bash
bijux-dag commands
```

- Capture the stable command inventory in JSON:

```bash
bijux-dag --json commands
```

### Help

```text
Usage: commands [OPTIONS]

Options:
      --groups

      --lane <LANES>
          [possible values: stable, experimental, simulated, internal]

  -h, --help
          Print help
```

## `plan`

### Examples

- Preview one run layout before execution:

```bash
bijux-dag plan explain ${GRAPH} --out ${RUNS_ROOT}
```

- Capture planning diagnostics in JSON:

```bash
bijux-dag --json plan diagnostics ${GRAPH}
```

### Help

```text
Usage: plan <COMMAND>

Commands:
  explain
  diagnostics
  diff
  equivalence
  closure
  backfill
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `plan explain`

#### Examples

- Preview run layout, path bindings, and scheduler estimates:

```bash
bijux-dag plan explain ${GRAPH} --out ${RUNS_ROOT} --run-id tutorial-run
```

- Capture the same planning payload in JSON:

```bash
bijux-dag --json plan explain ${GRAPH} --out ${RUNS_ROOT} --run-id tutorial-run
```

#### Help

```text
Usage: explain [OPTIONS] <DAGS>...

Arguments:
  <DAGS>...

Options:
      --out <OUT>

      --run-id <RUN_ID>

      --cache-dir <CACHE_DIR>

      --absolute-path-policy <ABSOLUTE_PATH_POLICY>
          [default: allow-literal]
          [possible values: allow-literal, deny-literal]

      --jobs <JOBS>
          [default: 1]

      --cpu-budget <CPU_BUDGET>

      --memory-budget-mb <MEMORY_BUDGET_MB>

      --gpu-device-budget <GPU_DEVICE_BUDGET>

      --resource-capacity <name=count>
          declare a named runtime capacity as <name=count>; repeat for resources such as license tokens or database slots

      --from-node <FROM_NODE>

      --to-node <TO_NODE>

      --select <SELECT>

      --exclude <EXCLUDE>

      --dependency-closure

  -h, --help
          Print help
```

### `plan diagnostics`

#### Examples

- Inspect planner diagnostics for one graph:

```bash
bijux-dag plan diagnostics ${GRAPH}
```

- Capture planner diagnostics in JSON:

```bash
bijux-dag --json plan diagnostics ${GRAPH}
```

#### Help

```text
Usage: diagnostics <DAGS>...

Arguments:
  <DAGS>...

Options:
  -h, --help
          Print help
```

### `plan diff`

#### Examples

- Compare two graph revisions through the planner surface:

```bash
bijux-dag plan diff ${GRAPH_A} ${GRAPH_B}
```

- Emit the planner diff in JSON:

```bash
bijux-dag --json plan diff ${GRAPH_A} ${GRAPH_B}
```

#### Help

```text
Usage: diff <BEFORE> <AFTER>

Arguments:
  <BEFORE>

  <AFTER>

Options:
  -h, --help
          Print help
```

### `plan equivalence`

#### Examples

- Ask whether two graph files still execute the same logical workflow:

```bash
bijux-dag plan equivalence ${GRAPH_A} ${GRAPH_B}
```

- Capture the equivalence decision in JSON:

```bash
bijux-dag --json plan equivalence ${GRAPH_A} ${GRAPH_B}
```

#### Help

```text
Usage: equivalence <BEFORE> <AFTER>

Arguments:
  <BEFORE>

  <AFTER>

Options:
  -h, --help
          Print help
```

### `plan closure`

#### Examples

- Resolve the upstream and downstream closure for a selected node set:

```bash
bijux-dag plan closure ${GRAPH} --select id:publish
```

- Emit the closure result in JSON:

```bash
bijux-dag --json plan closure ${GRAPH} --select id:publish
```

#### Help

```text
Usage: closure [OPTIONS] <DAGS>...

Arguments:
  <DAGS>...

Options:
      --select <SELECT>

  -h, --help
          Print help
```

### `plan backfill`

#### Examples

- Plan a backfill window with explicit partition keys:

```bash
bijux-dag plan backfill --window-start-unix-ms 1711929600000 --window-end-unix-ms 1712016000000 --partition-key tenant=atlas --partition-key region=se
```

- Emit the backfill plan in JSON:

```bash
bijux-dag --json plan backfill --window-start-unix-ms 1711929600000 --window-end-unix-ms 1712016000000 --partition-key tenant=atlas
```

#### Help

```text
Usage: backfill [OPTIONS] --window-start-unix-ms <WINDOW_START_UNIX_MS> --window-end-unix-ms <WINDOW_END_UNIX_MS>

Options:
      --window-start-unix-ms <WINDOW_START_UNIX_MS>

      --window-end-unix-ms <WINDOW_END_UNIX_MS>

      --partition-key <PARTITION_KEY>

  -h, --help
          Print help
```

## `run`

### Examples

- Execute one graph with explicit runtime inputs:

```bash
bijux-dag run ${GRAPH} --out ${RUNS_ROOT} --input source=artifacts/bijux-dag/input.txt --progress compact
```

- Run the same workflow in machine-readable mode with streamed progress snapshots:

```bash
bijux-dag --json run ${GRAPH} --out ${RUNS_ROOT} --input source=artifacts/bijux-dag/input.txt --progress compact
```

### Help

```text
Usage: run [OPTIONS] --out <OUT> <DAGS>...

Arguments:
  <DAGS>...

Options:
      --out <OUT>

      --input <INPUT>

      --inputs-file <INPUTS_FILE>

      --run-id <RUN_ID>

      --resume-run <RESUME_RUN>
          resume an existing run directory by run id, reusing only nodes whose persisted outputs still match their recorded evidence

      --resume-failure-mode <RESUME_FAILURE_MODE>
          choose whether nodes that cannot be safely reused are rerun or rejected during resume

          [default: rerun-incomplete]
          [possible values: rerun-incomplete, reject-incomplete]

      --latest <LATEST>

      --jobs <JOBS>
          [default: 1]

      --cpu-budget <CPU_BUDGET>

      --memory-budget-mb <MEMORY_BUDGET_MB>

      --gpu-device-budget <GPU_DEVICE_BUDGET>

      --resource-capacity <name=count>
          declare a named runtime capacity as <name=count>; repeat for resources such as license tokens or database slots

      --node-timeout-ms <NODE_TIMEOUT_MS>

      --run-timeout-ms <RUN_TIMEOUT_MS>

      --run-timeout-behavior <RUN_TIMEOUT_BEHAVIOR>
          [default: finish-running]
          [possible values: finish-running, cancel-running]

      --deny-network
          deny declared network effects; shell execution does not firewall sockets, and container execution only enforces this when the runtime can honor a no-network mode

      --deny-env
          deny declared environment effects; this is a policy gate over declared DAG effects, not a syscall sandbox over arbitrary process reads

      --deny-clock
          deny declared clock effects; this does not virtualize wall clock access inside spawned processes

      --clean-env
          run with the curated bijux environment instead of inheriting the full parent environment; this shapes environment variables only

      --hermetic
          enable the best-effort local policy profile by forcing --deny-network, --deny-clock, and --clean-env; this does not claim syscall sandboxing or host filesystem isolation

      --select <SELECT>

      --exclude <EXCLUDE>

      --to-node <TO_NODE>

      --dependency-closure

      --materialize-inputs <MATERIALIZE_INPUTS>
          [default: copy]
          [possible values: copy, hardlink, symlink]

      --cache <CACHE>
          [default: off]
          [possible values: off, read, readwrite]

      --cache-dir <CACHE_DIR>

      --remote-cache-dir <REMOTE_CACHE_DIR>

      --absolute-path-policy <ABSOLUTE_PATH_POLICY>
          [default: allow-literal]
          [possible values: allow-literal, deny-literal]

      --preflight-only

      --explain-scheduling

      --progress <PROGRESS>
          show live progress for `bijux-dag run`; `compact` renders operator-readable updates on stderr in human mode and streams `dag.run.progress` JSON lines on stdout when `--json` is active

          [default: off]
          [possible values: off, compact]

      --backend <BACKEND>
          choose the node execution backend; `kubernetes` runs container nodes as Kubernetes Jobs through kubectl plus a shared persistent volume claim, and `slurm` submits nodes through sbatch and polls sacct until each job reaches a terminal state

          [default: local]
          [possible values: local, kubernetes, slurm]

      --kubernetes-namespace <KUBERNETES_NAMESPACE>
          [default: bijux]

      --kubernetes-volume-claim <KUBERNETES_VOLUME_CLAIM>

      --kubernetes-shared-root <KUBERNETES_SHARED_ROOT>

      --slurm-queue <SLURM_QUEUE>
          [default: general]

      --slurm-partition <SLURM_PARTITION>
          [default: cpu]

  -h, --help
          Print help
```

## `replay`

### Examples

- Replay one retained run into a new output root:

```bash
bijux-dag replay ${RUN_DIR} --out ${RUNS_ROOT}/replay
```

- Capture a replay proof request in JSON:

```bash
bijux-dag --json replay ${RUN_DIR} --out ${RUNS_ROOT}/replay --prove
```

### Help

```text
Usage: replay [OPTIONS] --out <OUT> [RUN_DIR]

Arguments:
  [RUN_DIR]

Options:
      --source-run-id <SOURCE_RUN_ID>
          resolve the replay source run by run id instead of passing a source run directory path

      --source-run-root <SOURCE_RUN_ROOT>
          root directory used when resolving --source-run-id; defaults to the replay output root when omitted

      --out <OUT>

      --dry-run

      --sandbox
          forbid replay outputs from being written inside the source run directory; this is a write-boundary check, not a process sandbox

      --prove

      --reuse-cache

      --cache <CACHE>
          [default: off]
          [possible values: off, read, readwrite]

      --jobs <JOBS>
          [default: 1]

      --run-id <RUN_ID>

      --cpu-budget <CPU_BUDGET>

      --memory-budget-mb <MEMORY_BUDGET_MB>

      --gpu-device-budget <GPU_DEVICE_BUDGET>

      --resource-capacity <name=count>
          declare a named runtime capacity as <name=count>; repeat for resources such as license tokens or database slots

      --deny-network
          deny declared network effects; shell execution does not firewall sockets, and container execution only enforces this when the runtime can honor a no-network mode

      --deny-env
          deny declared environment effects; this is a policy gate over declared DAG effects, not a syscall sandbox over arbitrary process reads

      --deny-clock
          deny declared clock effects; this does not virtualize wall clock access inside spawned processes

      --clean-env
          run with the curated bijux environment instead of inheriting the full parent environment; this shapes environment variables only

      --hermetic
          enable the best-effort local policy profile by forcing --deny-network, --deny-clock, and --clean-env; this does not claim syscall sandboxing or host filesystem isolation

      --from-node <FROM_NODE>

      --select <SELECT>

      --exclude <EXCLUDE>

      --dependency-closure

      --materialize-inputs <MATERIALIZE_INPUTS>
          [default: copy]
          [possible values: copy, hardlink, symlink]

      --remote-cache-dir <REMOTE_CACHE_DIR>

  -h, --help
          Print help
```

## `runs`

### Examples

- List retained runs under one root:

```bash
bijux-dag runs list --root ${RUNS_ROOT}
```

- Summarize retained run history in JSON:

```bash
bijux-dag --json runs summary --root ${RUNS_ROOT}
```

### Help

```text
Usage: runs <COMMAND>

Commands:
  list
  show
  inspect
  history
  id-explain
  tree
  timeline
  scheduler-checkpoint
  stop
  diff
  verify
  doctor
  explain-failure
  summary
  compare
  trend
  failures
  flakes
  diagnostics-bundle
  index
  help                  Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `runs list`

#### Examples

- Enumerate retained runs under one root:

```bash
bijux-dag runs list --root ${RUNS_ROOT}
```

- Emit the run list in JSON:

```bash
bijux-dag --json runs list --root ${RUNS_ROOT}
```

#### Help

```text
Usage: list --root <ROOT>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs show`

#### Examples

- Show compact status and timing for one retained run:

```bash
bijux-dag runs show ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit the compact run view in JSON:

```bash
bijux-dag --json runs show ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: show --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs inspect`

#### Examples

- Inspect one retained run by id:

```bash
bijux-dag runs inspect ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit structured run inspection in JSON:

```bash
bijux-dag --json runs inspect ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: inspect --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs history`

#### Examples

- Filter retained history by status and selection:

```bash
bijux-dag runs history --root ${RUNS_ROOT} --status failed --select id:publish
```

- Capture filtered retained history in JSON:

```bash
bijux-dag --json runs history --root ${RUNS_ROOT} --status failed
```

#### Help

```text
Usage: history [OPTIONS] --root <ROOT>

Options:
      --root <ROOT>

      --status <STATUS>

      --graph <GRAPH>

      --source <SOURCE>

      --offset <OFFSET>

      --limit <LIMIT>

      --select <SELECT>

  -h, --help
          Print help
```

### `runs id-explain`

#### Examples

- Explain how one retained run id resolves under a root:

```bash
bijux-dag runs id-explain ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit the run-id explanation in JSON:

```bash
bijux-dag --json runs id-explain ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: id-explain --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs tree`

#### Examples

- Render the node tree for one retained run:

```bash
bijux-dag runs tree ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit the retained run tree in JSON:

```bash
bijux-dag --json runs tree ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: tree --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs timeline`

#### Examples

- Inspect ordered execution events for one retained run:

```bash
bijux-dag runs timeline ${RUN_ID} --root ${RUNS_ROOT}
```

- Filter timeline events and emit them in JSON:

```bash
bijux-dag --json runs timeline ${RUN_ID} --root ${RUNS_ROOT} --node publish
```

#### Help

```text
Usage: timeline [OPTIONS] --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

      --node <NODE>

      --event <EVENT>

      --since-unix-ms <SINCE_UNIX_MS>

      --until-unix-ms <UNTIL_UNIX_MS>

  -h, --help
          Print help
```

### `runs scheduler-checkpoint`

#### Examples

- Inspect the retained scheduler checkpoint for one run:

```bash
bijux-dag runs scheduler-checkpoint ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit the retained scheduler checkpoint in JSON:

```bash
bijux-dag --json runs scheduler-checkpoint ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: scheduler-checkpoint --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs stop`

#### Examples

- Request a stop for one retained run id:

```bash
bijux-dag runs stop ${RUN_ID} --root ${RUNS_ROOT}
```

- Capture the stop request response in JSON:

```bash
bijux-dag --json runs stop ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: stop --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs diff`

#### Examples

- Compare two retained run directories inside the runs lane:

```bash
bijux-dag runs diff ${RUN_DIR_A} ${RUN_DIR_B}
```

- Emit the runs diff in JSON:

```bash
bijux-dag --json runs diff ${RUN_DIR_A} ${RUN_DIR_B} --mode semantic
```

#### Help

```text
Usage: diff [OPTIONS] <RUN_A> <RUN_B>

Arguments:
  <RUN_A>

  <RUN_B>

Options:
      --mode <MODE>
          [default: semantic]
          [possible values: summary, semantic, artifact, provenance, timing, policy, cache, raw]

      --node <NODE>

      --explain

  -h, --help
          Print help
```

### `runs verify`

#### Examples

- Verify one retained run by id:

```bash
bijux-dag runs verify ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit retained-run verification in JSON:

```bash
bijux-dag --json runs verify ${RUN_ID} --root ${RUNS_ROOT} --strict
```

#### Help

```text
Usage: verify [OPTIONS] --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

      --deep

      --strict

  -h, --help
          Print help
```

### `runs doctor`

#### Examples

- Diagnose one retained run by id:

```bash
bijux-dag runs doctor ${RUN_ID} --root ${RUNS_ROOT}
```

- Emit the run diagnosis in JSON:

```bash
bijux-dag --json runs doctor ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: doctor --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs explain-failure`

#### Examples

- Explain the first causal failure for one retained run:

```bash
bijux-dag runs explain-failure ${RUN_ID} --root ${RUNS_ROOT}
```

- Capture the same failure explanation in JSON:

```bash
bijux-dag --json runs explain-failure ${RUN_ID} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: explain-failure --root <ROOT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs summary`

#### Examples

- Summarize retained history under one root:

```bash
bijux-dag runs summary --root ${RUNS_ROOT}
```

- Emit the retained summary in JSON:

```bash
bijux-dag --json runs summary --root ${RUNS_ROOT}
```

#### Help

```text
Usage: summary --root <ROOT>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs compare`

#### Examples

- Compare two retained run ids under one root:

```bash
bijux-dag runs compare ${RUN_ID_A} ${RUN_ID_B} --root ${RUNS_ROOT}
```

- Emit the retained-run comparison in JSON:

```bash
bijux-dag --json runs compare ${RUN_ID_A} ${RUN_ID_B} --root ${RUNS_ROOT}
```

#### Help

```text
Usage: compare --root <ROOT> <RUN_A> <RUN_B>

Arguments:
  <RUN_A>

  <RUN_B>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs trend`

#### Examples

- Review one analytics point per retained run:

```bash
bijux-dag runs trend --root ${RUNS_ROOT}
```

- Emit the trend report in JSON:

```bash
bijux-dag --json runs trend --root ${RUNS_ROOT}
```

#### Help

```text
Usage: trend --root <ROOT>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs failures`

#### Examples

- Aggregate failed node kinds across retained history:

```bash
bijux-dag runs failures --root ${RUNS_ROOT}
```

- Capture the failure aggregation in JSON:

```bash
bijux-dag --json runs failures --root ${RUNS_ROOT}
```

#### Help

```text
Usage: failures --root <ROOT>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs flakes`

#### Examples

- Find graph fingerprints with mixed retained outcomes:

```bash
bijux-dag runs flakes --root ${RUNS_ROOT}
```

- Emit the flake report in JSON:

```bash
bijux-dag --json runs flakes --root ${RUNS_ROOT}
```

#### Help

```text
Usage: flakes --root <ROOT>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

### `runs diagnostics-bundle`

#### Examples

- Assemble a diagnostics bundle for one retained run:

```bash
bijux-dag runs diagnostics-bundle ${RUN_ID} --root ${RUNS_ROOT} --out ${DIAGNOSTICS_DIR}
```

- Capture diagnostics-bundle metadata in JSON:

```bash
bijux-dag --json runs diagnostics-bundle ${RUN_ID} --root ${RUNS_ROOT} --out ${DIAGNOSTICS_DIR}
```

#### Help

```text
Usage: diagnostics-bundle [OPTIONS] --root <ROOT> --out <OUT> <RUN_ID>

Arguments:
  <RUN_ID>

Options:
      --root <ROOT>

      --out <OUT>

      --redact

  -h, --help
          Print help
```

### `runs index`

#### Examples

- Rebuild or inspect the retained run index under one root:

```bash
bijux-dag runs index --root ${RUNS_ROOT}
```

- Emit the run index payload in JSON:

```bash
bijux-dag --json runs index --root ${RUNS_ROOT}
```

#### Help

```text
Usage: index --root <ROOT>

Options:
      --root <ROOT>

  -h, --help
          Print help
```

## `diff`

### Examples

- Compare two retained runs semantically:

```bash
bijux-dag diff ${RUN_DIR_A} ${RUN_DIR_B}
```

- Capture an explained semantic diff in JSON:

```bash
bijux-dag --json diff ${RUN_DIR_A} ${RUN_DIR_B} --mode semantic --explain
```

### Help

```text
Usage: diff [OPTIONS] <RUN_A> <RUN_B>

Arguments:
  <RUN_A>

  <RUN_B>

Options:
      --mode <MODE>
          [default: semantic]
          [possible values: summary, semantic, artifact, provenance, timing, policy, cache, raw]

      --node <NODE>

      --explain

  -h, --help
          Print help
```

## `explain`

### Examples

- Explain one retained run at a high level:

```bash
bijux-dag explain ${RUN_DIR}
```

- Explain one retained node in JSON:

```bash
bijux-dag --json explain ${RUN_DIR} --node publish
```

### Help

```text
Usage: explain [OPTIONS] <RUN_DIR>

Arguments:
  <RUN_DIR>

Options:
      --node <NODE>

  -h, --help
          Print help
```

## `verify`

### Examples

- Verify one retained run directory directly:

```bash
bijux-dag verify ${RUN_DIR}
```

- Capture the verification result in JSON:

```bash
bijux-dag --json verify ${RUN_DIR} --strict
```

### Help

```text
Usage: verify [OPTIONS] <RUN_DIR>

Arguments:
  <RUN_DIR>

Options:
      --deep

      --strict

  -h, --help
          Print help
```

## `doctor`

### Examples

- Inspect runtime, cache, and environment readiness:

```bash
bijux-dag doctor
```

- Capture the doctor report in JSON:

```bash
bijux-dag --json doctor
```

### Help

```text
Usage: doctor

Options:
  -h, --help
          Print help
```

## `cache`

### Examples

- Review cache statistics before a cleanup or replay decision:

```bash
bijux-dag cache stats --cache-dir ${CACHE_DIR}
```

- Verify cache integrity in machine-readable form:

```bash
bijux-dag --json cache verify --cache-dir ${CACHE_DIR}
```

### Help

```text
Usage: cache <COMMAND>

Commands:
  ls
  pack
  unpack
  gc
  verify
  explain
  stats
  prune-simulate
  diff
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help
```

### `cache ls`

#### Examples

- List cache entries under one cache root:

```bash
bijux-dag cache ls --cache-dir ${CACHE_DIR}
```

- List cache entries in JSON:

```bash
bijux-dag --json cache ls --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: ls [OPTIONS]

Options:
      --cache-dir <CACHE_DIR>

  -h, --help
          Print help
```

### `cache pack`

#### Examples

- Pack one cache entry for transfer or evidence capture:

```bash
bijux-dag cache pack ${NODE_FINGERPRINT} --out artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}
```

- Emit the packed-entry metadata in JSON:

```bash
bijux-dag --json cache pack ${NODE_FINGERPRINT} --out artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: pack [OPTIONS] --out <OUT> <NODE_FP>

Arguments:
  <NODE_FP>

Options:
      --out <OUT>

      --cache-dir <CACHE_DIR>

  -h, --help
          Print help
```

### `cache unpack`

#### Examples

- Restore one packed cache entry into a cache root:

```bash
bijux-dag cache unpack artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}
```

- Capture the unpack result in JSON:

```bash
bijux-dag --json cache unpack artifacts/bijux-dag/cache-entry.tar.zst --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: unpack [OPTIONS] <PACK>

Arguments:
  <PACK>

Options:
      --cache-dir <CACHE_DIR>

  -h, --help
          Print help
```

### `cache gc`

#### Examples

- Run cache garbage collection on one cache root:

```bash
bijux-dag cache gc --cache-dir ${CACHE_DIR}
```

- Capture the cleanup result in JSON:

```bash
bijux-dag --json cache gc --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: gc [OPTIONS]

Options:
      --cache-dir <CACHE_DIR>

  -h, --help
          Print help
```

### `cache verify`

#### Examples

- Verify cache integrity before relying on reuse:

```bash
bijux-dag cache verify --cache-dir ${CACHE_DIR}
```

- Verify cache integrity and emit JSON:

```bash
bijux-dag --json cache verify --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: verify [OPTIONS]

Options:
      --cache-dir <CACHE_DIR>

      --remote <REMOTE>

  -h, --help
          Print help
```

### `cache explain`

#### Examples

- Explain one cache key and the expected adapter contract:

```bash
bijux-dag cache explain --cache-dir ${CACHE_DIR} --key ${CACHE_KEY}
```

- Emit the cache explanation in JSON:

```bash
bijux-dag --json cache explain --cache-dir ${CACHE_DIR} --key ${CACHE_KEY}
```

#### Help

```text
Usage: explain [OPTIONS] --key <KEY>

Options:
      --cache-dir <CACHE_DIR>

      --key <KEY>

      --expected-adapter-id <EXPECTED_ADAPTER_ID>

      --expected-adapter-version <EXPECTED_ADAPTER_VERSION>

  -h, --help
          Print help
```

### `cache stats`

#### Examples

- Summarize cache occupancy and entry counts:

```bash
bijux-dag cache stats --cache-dir ${CACHE_DIR}
```

- Emit cache statistics in JSON:

```bash
bijux-dag --json cache stats --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: stats [OPTIONS]

Options:
      --cache-dir <CACHE_DIR>

  -h, --help
          Print help
```

### `cache prune-simulate`

#### Examples

- Preview which cache entries a cleanup policy would remove:

```bash
bijux-dag cache prune-simulate --cache-dir ${CACHE_DIR}
```

- Review the same cleanup simulation in JSON:

```bash
bijux-dag --json cache prune-simulate --cache-dir ${CACHE_DIR}
```

#### Help

```text
Usage: prune-simulate [OPTIONS]

Options:
      --cache-dir <CACHE_DIR>

  -h, --help
          Print help
```

### `cache diff`

#### Examples

- Compare two cache entries for drift:

```bash
bijux-dag cache diff --cache-dir ${CACHE_DIR} --key-a ${CACHE_KEY_A} --key-b ${CACHE_KEY_B}
```

- Emit the cache diff in JSON:

```bash
bijux-dag --json cache diff --cache-dir ${CACHE_DIR} --key-a ${CACHE_KEY_A} --key-b ${CACHE_KEY_B}
```

#### Help

```text
Usage: diff [OPTIONS] --key-a <KEY_A> --key-b <KEY_B>

Options:
      --cache-dir <CACHE_DIR>

      --key-a <KEY_A>

      --key-b <KEY_B>

  -h, --help
          Print help
```

## `version`

### Examples

- Print the product version and build identity:

```bash
bijux-dag version
```

- Emit version identity in JSON:

```bash
bijux-dag --json version
```

### Help

```text
Usage: version

Options:
  -h, --help
          Print help
```

