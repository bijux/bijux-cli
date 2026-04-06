# Runtime crate tests

This directory contains runtime behavior contract suites grouped by trust surface.

Modeled platform and product-facing runtime APIs live under `bijux_dag_runtime::simulated_platform`.

- semantic: deterministic runtime behavior
- adversarial: hostile or malformed conditions
- failure: explicit failure-path semantics
- replay: mismatch and equivalence behavior
- scheduler: readiness ordering and edge behavior
- policy: policy violation semantics
- cache: invalidation and poisoning resistance
- artifact: corruption and lineage checks
- cancellation: terminal cancellation behavior
- state-machine: terminal consistency checks
- recovery: continuation and checkpoint behavior
- import-export: manifest integrity expectations
- node-execution: dependency and retry behavior
- scheduler-determinism: stable ordering guarantees
