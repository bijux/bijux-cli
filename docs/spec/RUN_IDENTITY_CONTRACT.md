# Run Identity Contract

## Run identity

- `run_id` is the immutable identifier for a finalized run directory.
- `run_id` must remain stable for the lifetime of historical run artifacts.
- `run_id` explanation and ancestry surfaces are available through:
  - `dag runs id-explain <run_id> --root <runs_dir>`
  - `dag runs history --root <runs_dir>`

## Composition

`run_id` is runtime-assigned and persisted in `manifest.json`. It is not derived from mutable alias links.

## Ancestry fields

- `run_metadata.parent_run_id`
- `run_metadata.source_run_id`

## Immutability

Historical run content must not be mutated by alias updates such as `--latest`.
