# Argo Workflows Scenario Mapping

- source scenarios: `evidence/compare/scenarios/`
- mapping method: `docs/spec/COMPARISON_HARNESS_CONTRACT.md`

| bijux scenario | Argo Workflows mapping note |
| --- | --- |
| chain | step template sequence |
| diamond | DAG template with fan-out and join |
| retry-timeout | retryStrategy and activeDeadlineSeconds mapping |
| scheduler-tiny-tasks-overhead | controller scheduling pressure comparison |
