# HPC and DNA Boundary

## Ownership boundary

`bijux-dag` owns:

- canonical graph/run/artifact/replay semantics
- HPC adapter contract semantics used by DAG execution truth surfaces
- evidence and trust contracts for scheduler mapping and replay fidelity

`bijux-dna` owns:

- DNA product-specific orchestration and product workflows
- domain-specific optimization layers that must not redefine DAG meaning
- additional UX/workflow layers built on top of dag truth surfaces

## Rule

DNA extensions may add behavior, but must consume dag capability and identity contracts without forking semantics.
