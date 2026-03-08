# Schema Usage Inventory Report

## Primary schema roots

- `configs/schema/dag.schema.json`
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`
- `configs/schema/outputs_index.schema.json`
- `configs/schema/proof*.schema.json`
- `configs/schema/operator/*.schema.json`

## Verification surfaces

- Compatibility fixtures under `evidence/compat/`
- Frozen hashes under `configs/policy/stable_schema_hashes.json`
- Governance contracts under `crates/bijux-dev-dag/tests/schema_*contracts.rs`

## Operator-facing inventory

- `docs/reference/COMPATIBILITY_MATRIX_GENERATED.md`
- `docs/reports/foundation/json_command_schema_inventory_report.md`
