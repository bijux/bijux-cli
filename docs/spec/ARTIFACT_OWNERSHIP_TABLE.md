---
title: Artifact Ownership Table
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Artifact Ownership Table

| surface | owner | boundary |
| --- | --- | --- |
| run-dir verification and finalization markers | `bijux-dag-artifacts` | owns storage hardening semantics |
| runtime manifest and trace production | `bijux-dag-runtime` | produces content but does not own import/export policy |
| import and export envelopes | `bijux-dag-app` | owns operator contract surface |
| governance hardening checks | `bijux-dev` | owns maintainer policy enforcement |

## Related tests

- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`
