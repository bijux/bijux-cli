# DAG explain and preview workflows

## Validation explain

Use strict parsing and diagnostics from `bijux-dag-core` to produce actionable explanations:

- what field failed
- what contract was violated
- how to fix the graph

## Node execution explain

Use run artifacts to explain why a node did not run:

- selection filters
- dependency failures
- policy denials
- cancellation
- cache reuse decisions

## Run preview

Use compile + simulation + scheduler policies to preview:

- planned execution order
- dependency closure for partial reruns
- estimated concurrency
- likely blocked nodes due to policy or resources

## Graph visualization

Use timeline and graph visualization artifacts for topology overlays with status and cache signals.
