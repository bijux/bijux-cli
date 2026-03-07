# Run directory and import export hardening report

## Scope

Captures hardening evidence for run-directory integrity and import/export compatibility surfaces.

## Run-directory truth boundaries

Authoritative files:

- `manifest.json`
- `graph.snapshot.json`
- `nodes/<node_id>/trace.json`
- `outputs/index.json`

Derived files remain non-authoritative and may not override authoritative content.

## Verification guarantees

- standard verification tolerates missing optional files
- strict verification requires full authoritative set and supported manifest version
- malformed run directories fail with explicit diagnostics

## Import/export guarantees

- bundle structure validation is mandatory before import acceptance
- unsupported bundle versions fail explicitly
- `manifest-only` and `with-files` export modes remain semantically distinct
- imported runs preserve provenance source markers

## Corruption and truncation handling

- truncated bundles fail closed
- invalid outputs index and missing manifest version fixtures remain part of corruption checks

## Battle trust linkage

Run-directory and import/export hardening protects:

- `tp_run_dir_resilience`
- `tp_import_export_compatibility`

Both properties remain mapped in battle scenario trust policy.
