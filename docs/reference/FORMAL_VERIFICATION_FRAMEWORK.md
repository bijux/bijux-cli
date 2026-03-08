# Formal methods, invariants, and verification framework

## Invariant catalog

The platform maintains explicit invariants for:
- DAG compile and plan determinism
- scheduler idempotence and fairness
- run-state transitions
- artifact lineage and integrity
- policy enforcement finality

## Machine-checkable invariants

Core invariants are represented as machine-checkable definitions where feasible.

## Verification suite layers

- property-based testing suites
- model-based state-machine suites
- scheduler state-space checks
- HA failover and fencing harnesses

## Counterexample reporting

Invariant failures must emit a minimal counterexample report with reproducible steps and observed violation details.

## Replay and diff guarantees

Deterministic replay invariants and formal diff semantic specifications are first-class verification surfaces.

## Fuzzing and adversarial fixtures

Verification includes fuzzing for parsing/planning/scheduling/manifest decoding and adversarial fixtures for malformed bundles, lineage cycles, policy corruption, and split-brain timing.

## Verified core and maturity labels

Verified core scope prioritizes strongest guarantees for trust-critical subsystems.

Maturity labels:
- specified
- property-tested
- model-tested
- formally constrained

## CI and release gates

Invariant suites are separate gates from ordinary unit and integration tests.
