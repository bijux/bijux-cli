# Repository Capability Matrix

This matrix is a root-level summary only. Normative details live in:

- `docs/spec/CURRENT_IMPLEMENTED_CAPABILITIES.md`
- `docs/spec/MODELED_AND_FUTURE_SURFACES.md`
- `docs/reference/EXECUTION_SUPPORT_POLICY.md`

| Surface | Mode | Notes |
| --- | --- | --- |
| Local process execution | implemented | Deterministic runtime path in this repo |
| Container execution contract | simulated | Contract and fixtures; not production runtime support claim |
| Kubernetes execution | simulated | No production execution backend implementation in this repo |
| Batch/HPC execution | modeled/simulated | Modeled semantics and fixtures only |
| Remote distributed execution | modeled/simulated | Not a production execution mode in this repo |
| Replay/diff/inspect workflows | implemented | Operator and evidence suites enforce behavior |
| Evidence release verification | implemented | Blocking/advisory split with drift checks |
