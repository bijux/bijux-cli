# Cost observability reports

## Required dimensions

Cost reports should include breakdowns by:
- DAG
- tenant
- queue
- backend
- region
- artifact class

## Operator explainability

Every cost-sensitive placement decision should provide a concise explanation of:
- selected backend
- rejected alternatives
- governing trust/latency/locality constraints
- incremental cost impact

## Backfill cost governance

Backfill throttling balances urgency with current platform cost pressure.
