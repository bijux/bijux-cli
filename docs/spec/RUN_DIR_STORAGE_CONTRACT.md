# Run directory storage contract

## Scope

Defines canonical run directory structure, required files, and verification behavior.

## Required files

- `manifest.json`
- `outputs.index.json`
- `trace/`

## Finalization files

- `manifest.finalized.json`
- `.run-complete.json`

## Incomplete marker

- `.run-incomplete.json` is written when a run ends before finalization.

## Verification modes

- `standard`: required files and manifest parse checks.
- `strict`: standard checks plus `manifest_version` and finalization files.
