# Run History Contract

## Scope

Defines machine-readable run ancestry and history query behavior.

## Command surfaces

- `dag runs history --root <runs_dir>`
- `dag runs id-explain <run_id> --root <runs_dir>`
- `dag runs summary --root <runs_dir>`
- `dag runs doctor <run_id> --root <runs_dir>`

## Schema surfaces

- `configs/schema/operator/run_history.schema.json`
- `configs/schema/operator/run_id_explain.schema.json`

## Invariants

- History output must include `run_id`, `parent_run_id`, and `source_run_id`.
- History queries are read-only and must never mutate run artifacts.
- Missing manifests produce actionable but non-panicking diagnostics.
- Missing trace surfaces referenced by manifest counters must be reported by doctor output.
- History traversal order is deterministic (`run_id` lexical order).
- `latest` alias updates are advisory and must not mutate historical rows.

## Ancestry field mapping

- `parent_run_id`: replay parent linkage from source run identity.
- `source_run_id`: origin run used for replay/import lineage.

## Recovery

- See `docs/spec/RUN_HISTORY_CORRUPTION_RECOVERY.md` for corruption handling and operator recovery procedure.
