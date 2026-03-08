# CLI surface compatibility report

## Release baseline

- previous: `v0.1`
- current: `main`

## Stable command surfaces verified

- `dag validate`
- `dag show-effective-plan`
- `dag run`
- `dag runs inspect`
- `dag replay`
- `dag diff`
- `dag artifact-inspect`
- `dag verify`
- `dag fsck`
- `dag export`
- `dag import --verify-only`
- `dag runs list|show|history|timeline|tree`
- `dag capabilities`

## Compatibility status

- stable command names remain callable
- legacy aliases remain callable
- no removed stable surface detected in this cycle
