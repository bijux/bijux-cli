---
title: Multi-Run Analytics Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Multi-Run Analytics Contract

Multi-run analytics in `bijux-dag` provide read-only aggregation across retained
run directories. They exist to summarize historical evidence, not to create a
second source of truth.

## Scope

This contract governs the operator-facing analytics commands:

- `dag runs summary`
- `dag runs compare`
- `dag runs trend`
- `dag runs failures`
- `dag runs flakes`

It also governs their JSON command identifiers, their schema lockstep surface,
and the run-evidence files they are allowed to read.

## Authoritative inputs

The commands derive analytics from finalized run evidence under the selected
run root:

- `manifest.json`
- `run.snapshot.json`
- `outputs/index.json`
- `nodes/*/trace.json`
- `nodes/*/outputs/index.json`

When present, graph identity is derived from `manifest.json` and may fall back
to recorded snapshot material already stored in the run directory.

## Non-mutation rule

Multi-run analytics must never mutate authoritative run records. They may read
retained run directories and render derived JSON reports, but they must not
rewrite manifests, node traces, outputs indexes, or finalized run content as a
side effect of analytics queries.

## Command and schema surfaces

The stable machine-facing command identifiers are:

- `dag.runs.summary`
- `dag.runs.compare`
- `dag.runs.trend`
- `dag.runs.failures`
- `dag.runs.flakes`

The schema lockstep surface is:

`configs/dag/schema/operator/runs_analytics.schema.json`

## Report responsibilities

- `summary`: aggregate run count, retry count, cache-use signals, artifact
  totals, and status distribution
- `compare`: compare two named runs across status, retries, cache hits,
  artifacts, timing, graph fingerprint, execution fingerprint, graph input
  values, selected nodes, node statuses, output hashes, and the first
  meaningful divergence that can be proven from retained evidence
- `trend`: emit one ordered point per visible run with retry, cache, artifact,
  and status fields
- `failures`: aggregate failed node kinds across retained run traces
- `flakes`: detect graph fingerprints whose retained runs show more than one
  terminal status

## Related tests

- `crates/bijux-dag-app/tests/multi_run_analytics_contract.rs`
- `crates/bijux-dag-app/src/inspect/run_views.rs`
- `crates/bijux-dag-app/src/routes/runs_routes.rs`

## Versioning and change policy

Any incompatible change to the analytics commands, their JSON identifiers, or
their read-only evidence boundary must update this contract, the linked schema,
and the linked tests in the same change.
