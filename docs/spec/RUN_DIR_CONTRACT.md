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

- `manifest.json`
- `outputs.index.json`
- `trace/`
- `run.log.jsonl`

These entries carry authoritative run evidence and must not be treated as optional.

## Optional entries

- `provenance.json`
- `run.snapshot.json`
- `observability.timeline.json`
- `observability.events.json`
- `observability.metrics.json`

Optional entries may be absent in standard verification without making the run
directory structurally invalid.

## Derived artifacts (non-authoritative)

- `manifest.finalized.json`
- `.run-complete.json`
- `.run-incomplete.json`
- `run-log.index.json`
- `run.schema.json`

These artifacts are derived from authoritative evidence and verification rules.

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
