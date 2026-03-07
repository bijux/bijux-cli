# Run History Contract

## Scope

Defines machine-readable run ancestry and history query behavior.

## Command surfaces

- `dag runs history --root <runs_dir>`
- `dag runs id-explain <run_id> --root <runs_dir>`

## Schema surfaces

- `configs/schema/operator/run_history.schema.json`
- `configs/schema/operator/run_id_explain.schema.json`

## Invariants

- History output must include `run_id`, `parent_run_id`, and `source_run_id`.
- History queries are read-only and must never mutate run artifacts.
- Missing manifests produce actionable but non-panicking diagnostics.
