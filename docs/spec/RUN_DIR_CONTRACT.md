---
title: Run Directory Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Run Directory Contract

The run directory is the durable execution evidence envelope for one DAG run.

## Scope

This contract covers the finalized run-directory layout and the operator-facing
verification behavior expected by run-dir audit, import, export, and replay
surfaces.

## Required entries (authoritative)

Required root entries are:

- `manifest.json`
- `graph.snapshot.json`
- `outputs/index.json`
- `provenance.json`
- `lineage.snapshot.json`
- `observability.events.json`
- `observability.timeline.json`
- `run.log.jsonl`
- `run.schema.json`

Required node entries are:

- `nodes/<node_id>/trace.json`
- `nodes/<node_id>/attempts.json`
- `nodes/<node_id>/resolved_params.json`
- `nodes/<node_id>/inputs/index.json`
- `nodes/<node_id>/outputs/index.json`

These entries carry authoritative run evidence and must not be treated as
optional.

## Optional entries

- `manifest.finalized.json`
- `.run-complete.json`
- `.run-incomplete.json`
- `run.snapshot.json`
- `run-log.index.json`
- `run.audit.json`
- `scheduler.checkpoint.json`
- `failure-propagation.json`
- `observability.metrics.json`
- `observability.root-causes.json`
- `observability.graph-visualization.json`
- `observability.lineage-visualization.json`
- `promotions/index.json`
- `plan.json`

Optional entries may be absent in standard verification without making the run
directory structurally invalid.

## Derived artifacts (non-authoritative)

- `manifest.finalized.json`
- `.run-complete.json`
- `.run-incomplete.json`
- `run-log.index.json`
- `run.audit.json`
- `observability.metrics.json`
- `observability.root-causes.json`
- `observability.graph-visualization.json`
- `observability.lineage-visualization.json`

These artifacts are derived from authoritative evidence and verification rules.

## Optional retained plan surface

- `plan.json` may retain the lowered execution plan when a producer writes it
- standard local run snapshots do not currently retain `plan.json` by default
- planner identity is still preserved through `manifest.json`,
  `graph.snapshot.json`, and `run.snapshot.json`

## Verification behavior

- standard verification tolerates missing optional artifacts
- strict verification requires finalized evidence surfaces and stronger manifest completeness
- `dag verify --strict` is the operator-facing command that enforces the strict contract

## Related tests

- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`

## Versioning and change policy

Any incompatible change to required run-dir entries, verification behavior, or
the authoritative-versus-derived split must update this contract and the linked
tests in the same change.
