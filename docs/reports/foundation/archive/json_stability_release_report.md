# JSON stability release report

## Scope

Machine-readable JSON envelope and key command payload stability for release review.

## Stable envelope contract

All stable commands keep:

- `ok`
- `command`
- `data`
- `diagnostics`
- `error`

## Surfaces checked this cycle

- `dag validate --json`
- `dag run --json`
- `dag replay --json`
- `dag diff --json`
- `dag runs inspect --json`
- `dag artifact-inspect --json`
- `dag verify --json`
- `dag fsck --json`
- `dag export --json`
- `dag import --verify-only --json`
- `dag capabilities --json`

## Result

- no envelope-breaking drift
- no required key removal detected on covered surfaces
