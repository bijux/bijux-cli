# Deterministic Pipeline Control

Bijux-dag aims to provide deterministic control over computation graphs: identical graph semantics, runtime policy, and environment constraints should produce comparable execution history and explainable differences.

## What is guaranteed

- Canonical graph hashing and canonical graph byte emission.
- Run identity and ancestry surfaces (`runs history`, `runs id-explain`, `hash run`).
- Replay fidelity checks (`replay --prove`) and explicit mismatch diagnostics.
- Diff and explain surfaces for root-cause analysis (`diff`, `why-rerun`, `why-cache-missed`).

## What is not guaranteed

- Human-readable text output shape stability.
- Cross-backend equivalence where backend capabilities differ.
- Perfect reproducibility when required artifacts or declared environment inputs are missing.

## Operator workflow

1. Validate and canonicalize graph intent.
2. Execute with run-dir persistence.
3. Replay with proof output when reproducibility matters.
4. Diff runs and inspect root-cause summaries for divergence.
5. Verify run integrity with `dag verify` or `dag fsck`.
