# Test trust contract

## Scope

This contract defines minimum runtime trust evidence requirements.

## Required evidence classes

- semantic
- adversarial
- failure
- replay mismatch
- scheduler edge behavior
- policy violation
- cache poisoning defense
- artifact corruption handling
- cancellation terminal behavior
- state machine consistency
- recovery behavior
- import/export manifest checks
- node execution behavior
- scheduler determinism

## Catalog source

Required suites are listed in `crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json`.

## Enforcement

Control-plane suite `test-trust-foundation` verifies contract docs and catalog-backed files exist.
