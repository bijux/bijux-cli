# Audit Public API Surface Changes

## Current release highlights

- added `dag prove` command surface
- added `dag proof-summary` command surface
- extended `dag export` with provenance and redaction controls

## Verification

Public API/report contracts in `crates/bijux-dev-dag/tests` enforce presence and wording consistency for these surfaces.
