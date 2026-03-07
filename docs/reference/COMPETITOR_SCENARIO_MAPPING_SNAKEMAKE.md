# Snakemake Scenario Mapping

- source scenarios: `evidence/compare/scenarios/`
- mapping method: `docs/spec/COMPARISON_HARNESS_CONTRACT.md`

| bijux scenario | Snakemake mapping note |
| --- | --- |
| chain | rule dependency chain |
| diamond | rule fan-out with merged target |
| determinism | repeatability under same input and environment |
| failure-diagnostics | failed rule diagnostics and logs |
