# Batch Backend Simulation Boundary

## Current truth

- Batch/HPC and Kubernetes scheduler behavior is represented by simulated contract surfaces.
- Simulation is used to verify invariants for capability declarations, replay safety, and failure classification.
- Production scheduler control-plane integration remains out of scope for this runtime kernel.

## Guardrails

- Unsupported scheduler-only features must be rejected, never silently downgraded.
- Simulated backend surfaces must stay labeled as simulated in support matrices and capability reports.
- Any graduation from simulated to implemented requires explicit contract and benchmark evidence updates.
