# CLI JSON Compatibility Report

## Scope

Tracks machine-readable output compatibility between adjacent releases for stable CLI command surfaces.

## Baseline compared

- Previous baseline: `v0.1`
- Candidate baseline: `main` (working tree)

## Covered commands

- `dag validate --json`
- `dag run --json`
- `dag replay --json`
- `dag diff --json`
- `dag explain --json`
- `dag status --json`
- `dag verify --json`
- `dag hash graph --json`
- `dag hash run --json`
- `dag hash artifact --json`
- `dag capabilities --json`
- `dag fsck --json`

## Compatibility rules

- Top-level envelope fields (`ok`, `command`, `data`, `diagnostics`) are stable.
- Existing `data` keys are not removed without a compatibility decision record.
- Additive `data` keys are allowed.

## Current result

- No envelope-breaking changes detected in covered command surfaces.
- New additive command surfaces: `dag hash run`, `dag fsck`.
