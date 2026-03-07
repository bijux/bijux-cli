# Prefect Scenario Mapping

- source scenarios: `evidence/compare/scenarios/`
- mapping method: `docs/spec/COMPARISON_HARNESS_CONTRACT.md`

| bijux scenario | Prefect mapping note |
| --- | --- |
| chain | flow with ordered tasks |
| retry-timeout | retry and timeout on task definitions |
| cache-reuse-shape | task result caching semantics |
| operator-inspectability | flow run and task run inspection |
