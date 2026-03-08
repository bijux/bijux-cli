# Explainability Completeness Report

## Coverage scope

Advanced explainability completeness is validated across:

- node-level explanations
- scheduler and replay decision explanations
- cache hit and miss explanations
- artifact lineage and dependency-chain explanations
- environment drift and backend capability mismatch explanations

## Completeness anchors

- `crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs`
- `crates/bijux-dev-dag/tests/advanced_explainability_completion_contracts.rs`
- `evidence/cache/explainability/regression_corpus.json`
