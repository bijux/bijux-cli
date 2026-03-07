# Runtime test trust audit

## Scope

Runtime test trust covers executable contract tests under `crates/bijux-dag-runtime/tests`.

## Classification

- `semantic`: verifies normative runtime behavior and invariants.
- `adversarial`: stresses malformed, hostile, or inconsistent inputs.
- `failure`: verifies error classification and degraded behavior.
- `replay`: validates deterministic equivalence and mismatch detection.
- `scheduler`: validates ordering, fairness, and edge readiness decisions.
- `state-machine`: validates terminal and transition constraints.
- `recovery`: validates partial-state continuation semantics.
- `artifact`: validates artifact integrity and corruption handling.
- `cache`: validates proof checks and poisoning resistance.
- `policy`: validates policy denial and violation semantics.

## Shallow-test policy

A runtime test is shallow when it only checks construction or serialization without checking behavior. Shallow runtime tests are not accepted for normative contracts.

## Duplicate-test policy

If two tests assert the same contract, one test must be removed and the retained test must own the contract reference.

## Critical trust surfaces

- deterministic scheduling and tie-breaks
- retry timeout and cancellation terminal behavior
- dependency and recovery correctness
- cache proof validation and invalidation boundaries
- artifact integrity and corruption rejection
- replay mismatch detection
- policy violation classification

## Ownership

Runtime maintainers own this audit and update it with each trust-surface change.
