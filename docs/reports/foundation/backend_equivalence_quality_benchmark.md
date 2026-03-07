# Backend Equivalence Quality Benchmark

This report focuses on equivalence quality, not throughput speed.

## Dimensions
- semantic equivalence rate across backend pairs
- downgrade frequency and classified reasons
- cache equivalence correctness on equivalent workloads
- replay fidelity downgrade incidence across backend transitions

## Evidence inputs
- `evidence/compat/backend_equivalence/local_vs_k8s.json`
- `evidence/compat/backend_equivalence/local_vs_hpc.json`
- `evidence/compat/backend_equivalence/local_vs_remote.json`
- `evidence/compat/backend_equivalence/k8s_vs_imported_local_replay.json`
- `evidence/compat/backend_equivalence/hpc_vs_imported_local_replay.json`
