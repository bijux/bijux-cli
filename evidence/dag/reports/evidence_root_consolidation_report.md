# Evidence Root Consolidation Report

Date: 2026-03-07

## Before

- Root carried competing scenario ownership:
  - `examples/`
  - `benchmarks/scenarios`
  - `comparisons/scenarios`
  - `tests/e2e/fixtures`

## After

- Root pillars are stable and minimal:
  - `crates/`
  - `docs/`
  - `configs/dag/`
  - `evidence/`
  - `make/`
- `tests/` remains code-only with contract docs; no canonical scenario JSON ownership.

## Integrity checks

- `bijux-dev-dag verify evidence-drift`
- `bijux-dev-dag verify evidence-consumers`
- `bijux-dev-dag verify evidence-foundation`
