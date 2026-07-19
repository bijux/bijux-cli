# Run Directory Import Export Hardening Report

## Purpose

This report records the repository surfaces that currently harden run-directory
storage, verification, lifecycle, and import/export compatibility.

## Guarded surfaces

- `docs/spec/RUN_DIR_STORAGE_CONTRACT.md`
- `docs/spec/RUN_DIR_CONTRACT.md`
- `docs/spec/RUN_DIR_OWNERSHIP.md`
- `docs/spec/IMPORT_EXPORT_CONTRACT.md`
- `docs/spec/ARTIFACT_OWNERSHIP_TABLE.md`
- `docs/spec/ARTIFACT_LIFECYCLE.md`
- `crates/bijux-dag-artifacts/src/storage/hardening.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs`
- `configs/dag/schema/operator/run_verify_report.schema.json`
- `evidence/compat/export_bundle/v0_1_supported/bundle.json`
- `evidence/compat/export_bundle/unsupported_older_version/bundle.json`
- trust properties: `tp_run_dir_resilience`, `tp_import_export_compatibility`

## Current hardening stance

- run-dir structure must distinguish authoritative evidence from derived artifacts
- strict verification must require stronger finalization evidence than standard verification
- import/export compatibility must stay explicit about bundle versioning and export modes
- corruption fixtures remain governed evidence, not ad hoc test inputs
