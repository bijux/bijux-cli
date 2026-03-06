# Platform maturity scorecard

This scorecard tracks readiness across product subsystems and extension ecosystem maturity.

## Readiness dimensions

- engine
- scheduler
- artifacts
- observability
- API
- infrastructure
- ecosystem

## Current scorecard contract

Scores are represented as `0..100` readiness percentages and may be averaged for an overall maturity signal.

| Dimension | Score | Notes |
| --- | ---: | --- |
| Engine | 80 | Deterministic planning, scheduling, and replay contracts are established. |
| Scheduler | 78 | Enterprise scheduling contracts and simulation fixtures are in place. |
| Artifacts | 82 | Artifact platform contracts, lineage, and conformance suites are present. |
| Observability | 79 | Deep diagnostics and explainability contracts are implemented. |
| API | 70 | Control-plane API models and MVP boundaries are defined. |
| Infrastructure | 72 | Backend capability and deployment seam contracts are defined. |
| Ecosystem | 68 | Plugin boundaries and trust/isolation contracts are defined with lifecycle model. |

Computed overall maturity: `75`.

## Advancement gates

- all official plugins pass plugin conformance suites
- API compatibility rules are enforced in release checks
- ecosystem docs distinguish official and community support expectations
- extension discovery inventory is available in operator diagnostics
