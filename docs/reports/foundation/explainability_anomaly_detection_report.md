# Explainability Anomaly Detection Report

## Scope

Anomaly detection for explain outputs focuses on schema mismatches, missing reason groups, unstable ordering, and contradictory explanation payloads.

## Detection anchors

- explain schema lockstep tests in `json_output_governance_contracts.rs`
- explain ordering and stress checks in `explain_surface_completion_contracts.rs`
- advanced explainability anomaly coverage in `advanced_explainability_completion_contracts.rs`

## Governance references

- `configs/suites/advanced_explainability_regression.json`
- `evidence/cache/explainability/regression_corpus.json`
