# Cache Explainability Integration Report

## Scope

Cache integrity outcomes must remain explainable through operator-facing miss/hit reasoning surfaces.

## Explainability anchors

- `docs/spec/EXPLAIN_SURFACES_CONTRACT.md`
- `docs/spec/CACHE_CONTRACT.md`
- `evidence/cache/explain/regression_corpus.json`
- `crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs`

## Integration expectation

Cache corruption and invalidation causes are visible in explain outputs and remain stable across repeated inspections.
