# Test philosophy

## Goal

Tests are evidence for normative behavior, not implementation snapshots.

## Principles

- Prefer contract-level assertions over structural assertions.
- Prefer deterministic assertions over timing-sensitive assumptions.
- Prefer explicit failure-path checks for every normative success path.
- Keep one primary owner test per contract surface.
- Keep fixtures executable and versioned with the test that uses them.

## Runtime trust strategy

- semantic contract tests define expected behavior
- adversarial tests prevent unsafe assumptions
- failure-path tests enforce explicit error classes
- replay tests enforce deterministic equivalence expectations
- scheduler tests enforce deterministic ordering and fairness

## Promotion policy

A test is promoted to critical when it guards deterministic execution, state transition safety, cache proof correctness, artifact integrity, policy safety, or replay equivalence.
