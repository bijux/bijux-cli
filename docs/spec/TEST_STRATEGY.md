# Test strategy

## Testing pyramid

1. unit
2. module
3. contract
4. integration
5. end-to-end
6. replay
7. fault-injection
8. performance
9. compatibility

## Crate ownership and forbidden test types

- `bijux-dag-core`
  - owns: unit, module, contract, compatibility fixtures
  - forbidden: end-to-end, process-spawn runtime execution
- `bijux-dag-artifacts`
  - owns: contract, integration, corruption/fault artifact tests
  - forbidden: scheduler behavior tests
- `bijux-dag-runtime`
  - owns: state-machine, execution contract, cache, replay, fault tests
  - forbidden: direct CLI UX snapshot tests
- `bijux-dag-app`
  - owns: command integration and error-path tests
  - forbidden: runtime internal state transition tests
- `bijux-dag-cli`
  - owns: binary wiring and exit-code mapping tests
  - forbidden: runtime planning/execution internals
- `bijux-dev-dag`
  - owns: governance, policy, contract, release discipline checks
  - forbidden: product runtime behavior tests

## Universal rules

- Only e2e tests may shell out to production binaries.
- Every public command requires one integration test and one error-path test.
- Every schema requires positive and negative fixtures.
- Runtime state transitions require explicit transition coverage.
- Cache behavior requires `off`, `read`, and `readwrite` mode coverage.
