# Artifact Bundle Format v1

## Identifier

`artifact-bundle/v1`

## Required fields

- `bundle_version`: `export-bundle/v0.1`
- `format`: `artifact-bundle/v1`
- `outputs`

## Optional fields

- `files`
- `provenance`

## Invariants

- If `files` is present, each file entry must map to a stable output path key.
- If exported with `without-artifacts`, `outputs` must be an empty map and `files` must be null.
