# Backend Non-Equivalences

Known non-equivalences are explicit and must never be silently approximated.

- Kubernetes-only host namespace semantics (`hostNetwork`, `hostPID`) are rejected.
- Scheduler-specific HPC features outside contract support are rejected.
- Remote worker protocol mismatches downgrade portability and equivalence status.
- Missing artifact payloads downgrade replay fidelity even when run metadata matches.

These rules are governed by:
- `docs/spec/BACKEND_EQUIVALENCE_CONTRACT.md`
- `docs/spec/K8S_ADAPTER_CONTRACT.md`
- `docs/spec/HPC_ADAPTER_CONTRACT.md`
- `docs/spec/WORKER_PROTOCOL_CONTRACT.md`
