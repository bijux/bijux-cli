# CPU and Memory Budget Model

Runtime budget controls:

- `jobs`: upper bound on parallel dispatch width
- `cpu_budget`: aggregate CPU budget for batch scheduling
- node resource request: `cpu` and `mem_mb` contracts from graph node resources

Dispatch rule:

- a node is dispatch-eligible only if adding it does not exceed current CPU budget
- blocked nodes remain visible in scheduler blocked-by-budget diagnostics

