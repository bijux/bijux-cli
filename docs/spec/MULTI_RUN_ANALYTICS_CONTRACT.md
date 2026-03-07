# Multi-Run Analytics Contract

## Scope
Multi-run analytics are supported over an explicit runs root directory.
Commands are read-only and never mutate authoritative run records.

## Minimal analytics surfaces
- `dag runs summary --root <runs_dir>`
- `dag runs compare <run_a> <run_b> --root <runs_dir>`
- `dag runs trend --root <runs_dir>`
- `dag runs failures --root <runs_dir>`
- `dag runs flakes --root <runs_dir>`

## Run index model
- Run history is the set of direct child directories under `--root`.
- Each run directory is treated as authoritative local evidence.
- Analytics are derived views over that authoritative set.

## Incomplete history behavior
- Missing optional artifacts are tolerated where possible.
- Corrupt JSON is treated as unknown/null fields, not process crash.
- Commands keep returning partial aggregates when enough evidence exists.

## Aggregated output schema
JSON output for analytics commands must conform to:
- `configs/schema/operator/runs_analytics.schema.json`

## Determinism and replay signals
`dag runs summary` emits report sections:
- determinism report
- cache usefulness report
- replay equivalence report
- failure distribution report

These reports summarize observed history and do not assert stronger guarantees than the evidence supports.

## Data authority boundary
- Authoritative: run manifests, snapshots, traces, outputs indexes.
- Derived: analytics aggregates and trend series.
- Rule: analytics must never rewrite authoritative run files.
