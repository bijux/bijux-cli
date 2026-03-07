# Scheduler Fairness and Determinism

The runtime scheduler is deterministic by default:

- ready queue tie-break uses lexicographic node id ordering
- dispatch decisions are reproducible for identical inputs and budgets

Fairness tradeoff:

- deterministic mode prioritizes repeatability and replay explainability
- throughput mode prioritizes queue drain rate while preserving contract invariants

