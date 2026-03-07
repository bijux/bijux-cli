# Kubernetes Support Matrix

| Surface | Status | Evidence source |
| --- | --- | --- |
| Resource/timeout/retry/cancel mapping | contract-implemented | `backend_cluster_contracts` |
| Failure normalization | contract-implemented | `backend_cluster_contracts` |
| Secret/config injection validation | contract-implemented | `backend_cluster_contracts` |
| Watch-event determinism and reconnect reconciliation | contract-implemented | `backend_cluster_contracts` |
| Capability report (`dag capabilities --backend kubernetes`) | implemented | CLI/app contract tests |
| Kubernetes execution backend | simulated | `docs/reference/EXECUTION_SUPPORT_POLICY.md` |
| Startup / many-small-node / large-artifact benchmarks | contract-baseline | `docs/reports/foundation/k8s_adapter_benchmarks.md` |

## Support rule

This matrix is authoritative only when linked contract suites are green.
