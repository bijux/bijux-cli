# HPC Support Matrix

| Surface | Status | Evidence source |
| --- | --- | --- |
| Queue/partition mapping | contract-implemented | `backend_cluster_contracts` |
| Timeout-to-walltime mapping | contract-implemented | `backend_cluster_contracts` |
| Retry precedence (scheduler vs bijux) | contract-implemented | `backend_cluster_contracts` |
| Scratch/staging semantics | contract-implemented | `backend_cluster_contracts` |
| Failure normalization (queue/account/walltime/preemption) | contract-implemented | `backend_cluster_contracts` |
| HPC capability report (`dag capabilities --backend hpc`) | implemented | CLI/app contract tests |
| HPC execution backend | simulated | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| HPC benchmark baselines | contract-baseline | `docs/reports/foundation/hpc_adapter_benchmarks.md` |

## Support rule

This matrix is authoritative only when linked contract suites are green.
