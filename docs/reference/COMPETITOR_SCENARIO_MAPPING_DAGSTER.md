# Dagster Scenario Mapping

- source scenarios: `evidence/compare/scenarios/`
- mapping method: `docs/spec/COMPARISON_HARNESS_CONTRACT.md`

| bijux scenario | Dagster mapping note |
| --- | --- |
| chain | linear op graph |
| diamond | fan-out and fan-in job graph |
| failure-propagation | op failure propagation to downstream steps |
| determinism | materialization and rerun consistency checks |
