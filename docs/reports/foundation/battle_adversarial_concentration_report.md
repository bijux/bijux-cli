# Battle Adversarial Concentration Report

## Purpose

Concentrate battle proof on adversarial trust scenarios that stress correctness boundaries rather than scenario-count growth.

## Added adversarial scenarios

- adversarial-concurrency-retry-determinism
- adversarial-post-success-artifact-corruption
- adversarial-cache-proof-corruption-plausible-outputs
- adversarial-replay-semantic-drift-detection
- adversarial-import-export-semantic-loss-rejected
- adversarial-operator-only-recovery-path
- adversarial-policy-denial-blocks-unsafe-execution
- adversarial-missing-outputs-superficial-success-rejected
- adversarial-tie-break-stability-under-contention
- adversarial-cancel-retry-bookkeeping-integrity
- adversarial-path-escape-via-declared-outputs-blocked
- adversarial-env-leakage-via-adapters-blocked
- adversarial-partial-run-dir-not-finalized
- adversarial-imported-runs-remain-visible

## Concentration actions

- removed overlapping duplicate scenario registry entry for `e2e_matrix`.
- classified strongest adversarial subset as release-blocking in `configs/policy/battle_release_blocking_subset.json`.
- retained two scenarios as advisory to avoid release-set bloat while preserving recovery/import visibility coverage.

## Quality posture

Battle governance is centered on trust-property coverage with explicit invariant bundles, operator-visible proof surfaces, and replay/cache implications for every adversarial scenario.
