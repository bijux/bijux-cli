# Repository Root Topology Before And After Evidence Consolidation

Date: 2026-03-07
Owner: bijux-dev-dag

## Before
Proof assets were split across multiple top-level roots:
- `examples/`
- `benchmarks/`
- `comparisons/`
- `tests/` scenario trees
- `crates/*/tests/fixtures` ownership islands

## After
Proof assets are concentrated under `evidence/` and consumed by tests/benchmarks/comparisons as clients.

Retired proof roots:
- `examples/` removed
- `benchmarks/` removed
- `comparisons/` removed

Governing top-level pillars:
- `crates/`
- `docs/`
- `evidence/`
- `configs/`
- `make/`

Additional operational roots (`artifacts/`, `tests/`) remain for outputs and test code, not proof ownership.
