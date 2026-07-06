---
title: Artifact Lifecycle
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Artifact Lifecycle

Run artifacts move through governed stages from active execution evidence to
finalized and exported compatibility bundles.

## Lifecycle stages

1. runtime writes active evidence into the staging run directory
2. finalization copies `manifest.json` into `manifest.finalized.json`
3. completion or incompleteness is marked explicitly
4. verification audits structural completeness
5. export emits compatibility bundles in documented modes

## Related tests

- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`
