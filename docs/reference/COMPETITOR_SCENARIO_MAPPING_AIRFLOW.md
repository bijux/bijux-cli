# Airflow Scenario Mapping

- source scenarios: `evidence/compare/scenarios/`
- mapping method: `docs/spec/COMPARISON_HARNESS_CONTRACT.md`

| bijux scenario | Airflow mapping note |
| --- | --- |
| chain | DAG with linear task dependencies |
| diamond | DAG with fan-out and join |
| retry-timeout | retries and timeout via operator settings |
| replay-equivalence | rerun comparison requires explicit execution date handling |
