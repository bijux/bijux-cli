---
title: Observability And Diagnostics
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Observability And Diagnostics

Diagnose `bijux-dag` from retained evidence, not from a terminal message alone.
A process exit explains whether one command succeeded; a finalized run
directory records graph identity, node outcomes, artifacts, ordered events,
cache decisions, and integrity evidence.

## Choose The Addressing Model

The CLI exposes equivalent investigation paths for different operator inputs:

| What you have | Use |
| --- | --- |
| a retained root and run ID | `bijux-dag runs ... <RUN_ID> --root <RUNS_ROOT>` |
| an exact run directory | root commands such as `bijux-dag explain <RUN_DIR>` or `bijux-dag verify <RUN_DIR>` |
| no run yet, or an environment problem | `bijux-dag doctor` |

`bijux-dag doctor` checks runtime, cache, and environment readiness. It is not a
retained-run integrity verdict. Use `runs doctor` or `verify` for one run.

## Evidence-First Investigation

Start by fixing the evidence root and run identity:

```bash
RUNS_ROOT=./artifacts/runs
RUN_ID=<retained-run-id>
RUN_DIR="${RUNS_ROOT}/run-${RUN_ID}"

bijux-dag runs list --root "${RUNS_ROOT}"
bijux-dag runs show "${RUN_ID}" --root "${RUNS_ROOT}"
bijux-dag --json runs inspect "${RUN_ID}" --root "${RUNS_ROOT}"
```

`runs show` is a compact orientation view. `runs inspect` exposes structured
run, node, artifact, and compatibility information. Neither replaces integrity
verification.

Next reconstruct execution:

```bash
bijux-dag runs tree "${RUN_ID}" --root "${RUNS_ROOT}"
bijux-dag runs timeline "${RUN_ID}" --root "${RUNS_ROOT}"
bijux-dag runs scheduler-checkpoint "${RUN_ID}" --root "${RUNS_ROOT}"
```

- `tree` reads retained graph structure and dependency relationships.
- `timeline` reads normalized lifecycle order from
  `observability.timeline.json`, with compatibility fallback when required.
- `scheduler-checkpoint` explains one retained scheduling boundary, including
  ready, blocked, inflight, and completed state.

Use timeline filters to narrow a large run without discarding the original
evidence:

```bash
bijux-dag --json runs timeline "${RUN_ID}" \
  --root "${RUNS_ROOT}" \
  --node publish \
  --event failed
```

## Failure Diagnosis

For a failed run, identify the causal failure before inspecting every
downstream skip:

```bash
bijux-dag runs explain-failure "${RUN_ID}" --root "${RUNS_ROOT}"
bijux-dag --json explain "${RUNS_ROOT}/run-${RUN_ID}" --node publish
```

`runs explain-failure` separates the first causal failure from nodes affected
by propagation policy. `explain` gives a run- or node-level view by directory.
Retain the structured error code, category, exit status, node trace, attempt
records, and stderr together. Human message text alone is not a stable
classification surface.

Do not retry automatically until the failure class permits it:

| Class | First response |
| --- | --- |
| parse, schema, or validation | correct the graph or input; retrying unchanged input is not remediation |
| policy | change the request or governed policy deliberately |
| execution | inspect attempts, backend, timeout, and dependency propagation |
| IO or artifact integrity | preserve the run, then establish missing, corrupt, or unauthorized paths |
| replay or cache | compare identity, environment, lineage, and proof inputs |
| compatibility | inspect schema and release support before migration |
| internal or unknown | retain the complete structured payload and diagnostics bundle |

The public identifiers and ownership rules are in
[Error Codes](../interfaces/error-codes.md).

## Integrity Before Trust

Verify retained evidence before using it for release, replay, promotion, or
incident conclusions:

```bash
bijux-dag runs verify "${RUN_ID}" --root "${RUNS_ROOT}" --strict
bijux-dag runs doctor "${RUN_ID}" --root "${RUNS_ROOT}"
```

The direct-directory equivalent is:

```bash
bijux-dag verify "${RUN_DIR}" --strict
```

Use `--deep` when payload-level verification is required. A terminal
`completed` status does not prove that retained files still match hashes and
schemas. Conversely, an intentionally failed scenario can have internally
valid retained evidence.

Run doctor explains corruption, incompleteness, unsupported formats, and
compatibility problems. It should guide ownership; it must not be used to
reinterpret an unhealthy run as successful.

## Compare Without Losing Attribution

Use semantic comparison by default:

```bash
bijux-dag --json runs diff "${RUN_DIR_A}" "${RUN_DIR_B}" \
  --mode semantic \
  --explain
```

Available modes separate summary, semantic, artifact, provenance, timing,
policy, cache, and raw differences. Select the mode that owns the question.
Timing drift is not semantic drift, and byte-level difference is not proof that
two runs have different declared meaning.

For retained IDs under one root, `runs compare` provides status, retry, cache,
timing, artifact, and policy attribution. Use `runs trend`, `runs failures`,
and `runs flakes` only after verifying that the root contains comparable run
families; aggregation cannot repair inconsistent evidence.

## Support Bundle

Create a bounded support bundle without modifying the source run:

```bash
bijux-dag runs diagnostics-bundle "${RUN_ID}" \
  --root "${RUNS_ROOT}" \
  --out ./artifacts/diagnostics \
  --redact
```

The command writes a separate bundle. `--redact` reduces known sensitive
fields, but the operator must still inspect the bundle before sharing it.
Stdout, stderr, parameters, environment-derived data, and artifact paths may
carry confidential information.

Record the source run ID, source commit or release, command, exit status, and
bundle digest with incident evidence.

## Mutation Boundary

The investigation commands above are intended to read retained runs, except
for writing a separate diagnostics bundle. Keep these actions out of initial
triage:

- `runs stop` requests a state change for an active run;
- `runs index` may rebuild retained history indexing;
- repair and migration commands may write new evidence or change layout;
- cache prune, unpack, or garbage collection changes reusable state.

Capture a read-only evidence copy and complete verification before repair.
Never edit `manifest.json`, node traces, indexes, proofs, or timelines by hand
to make a run appear healthy.

## Source Authorities

- generated command syntax:
  `docs/bijux-dag/interfaces/generated-cli-reference.md`
- retained evidence model:
  `crates/bijux-dag-artifacts/src/storage/models.rs`
- run views and comparisons: `crates/bijux-dag-app/src/inspect/`
- replay and semantic diff: `crates/bijux-dag-app/src/replay/`
- integrity and run doctor:
  `crates/bijux-dag-app/src/inspect/integrity_service.rs`
- runtime event production:
  `crates/bijux-dag-runtime/src/diagnostics/`
- observability contracts:
  `crates/bijux-dag-runtime/tests/observability_contracts.rs`

## Next Reads

- [Failure Recovery](failure-recovery.md)
- [Run Evidence Layout](../interfaces/run-evidence-layout.md)
- [Operator Command Index](../interfaces/operator-command-index.md)
- [Reproducibility Model](../interfaces/reproducibility-model.md)
