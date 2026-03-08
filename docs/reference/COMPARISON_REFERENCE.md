# Comparison reference

Audience: maintainers.  
Owner: evidence governance.  
Status: stable.

## Evidence roots

- scenarios: `evidence/compare/scenarios/`
- baselines: `evidence/compare/baselines/`
- metadata: `evidence/compare/metadata.json`
- contract: `docs/spec/COMPARISON_HARNESS_CONTRACT.md`

## Usage rules

- Public comparison claims must reference committed evidence under `evidence/compare/`.
- Interpretations must be separated from facts in published summaries.
- Comparisons are valid only for committed harness definitions and versions.

## Scenario template

| Scenario | Bijux DAG | Airflow | Dagster | Prefect | Nextflow | Snakemake | Argo Workflows | Luigi | Raw evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| chain |  |  |  |  |  |  |  |  |  |
| diamond |  |  |  |  |  |  |  |  |  |
| retry-timeout |  |  |  |  |  |  |  |  |  |
| determinism |  |  |  |  |  |  |  |  |  |
| replay-equivalence |  |  |  |  |  |  |  |  |  |

## Known limitations

- Results are scenario-scoped and environment-sensitive.
- Engines have non-equivalent feature sets and defaults.
- Absolute performance numbers are not portable across hosts.
- Orchestration and observability models differ by design.

## Competitor mapping summary

| Bijux scenario | Airflow | Dagster | Prefect | Nextflow | Snakemake | Argo Workflows | Luigi |
| --- | --- | --- | --- | --- | --- | --- | --- |
| chain | DAG with linear task dependencies | linear op graph | flow with ordered tasks | channel-driven process sequence | rule dependency chain | step template sequence | ordered task requirements |
| diamond | DAG with fan-out and join | fan-out and fan-in job graph | flow with fan-out and merge | channel split and merge process shape | rule fan-out with merged target | DAG template with fan-out and join | branching dependencies with join task |
| retry-timeout | retries and timeout via operator settings | retry/failure behavior through op config | retry and timeout on task definitions | process retries with resume semantics | retry and failure handling by rule execution model | retryStrategy and activeDeadlineSeconds mapping | task failure and retry behavior through scheduler |
| replay-equivalence | explicit execution date handling for reruns | materialization/rerun consistency checks | cache reuse and rerun comparison | resume behavior compared with replay claims | repeatability under same inputs and environment | workflow resubmit and artifact comparison | rerun behavior through task state history |
