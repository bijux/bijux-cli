# Test trust coverage report

## Runtime test surfaces

- runtime test files (`crates/bijux-dag-runtime/tests/*.rs`): `80`
- test trust catalog fixture: present
- test trust ledger policy: present
- trust classes covered: semantic, adversarial, failure, replay, scheduler, policy, cache, artifact, cancellation, state machine, recovery, import/export, node execution, scheduler determinism, security, battle

## Cleanup posture

- must-never-break trust surfaces are governed in `configs/policy/test_trust_ledger.json`.
- low-value classes (`cosmetic`, `duplicate`) are tracked and currently empty by policy.
- snapshot assertions are restricted by explicit allowlist and forbidden-macro policy.

## Risk notes

- trust catalog and ledger both require periodic pruning to avoid silent overlap growth.
- semantic surface mappings should be updated in the same change as new high-stakes runtime behavior.
