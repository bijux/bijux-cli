# App Fast-Lane Skipped Scenarios With Reasons

generated_from: e2e ignored tests and lane rationale

| scenario | reason |
| --- | --- |
| `e2e_minimal_parse_validate_run_inspect_replay` | multiple spawned CLI invocations through cargo run |
| `e2e_diamond_outputs_and_manifest_totals` | end-to-end manifest/output accounting depth |
| `e2e_failure_downstream_behavior` | failure propagation and downstream semantics |
| `e2e_retry_accounting_present` | retry accounting path under runtime pressure |
| `e2e_timeout_error_classification` | timing-sensitive timeout classification |
| `e2e_missing_outputs_failure_handling` | failure-classification path with missing outputs |
| `e2e_cache_hit_second_run_and_invalidation` | multi-run cache invalidation workload |
