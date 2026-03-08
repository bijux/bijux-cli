# Dataset catalog query model

## Query filters

- `schema_ref`
- `owner`
- `freshness_max_minutes`
- `quality_state`

## Query behavior

Filtering is deterministic and composable. Results are only entries satisfying all provided filter constraints.

## Example usage

A downstream schedule may request:
- schema `schema/sales/v1`
- owner `analytics`
- freshness <= 60 minutes
- quality state `accepted`

This enables schedule readiness checks tied to data quality and freshness, not only DAG completion.
