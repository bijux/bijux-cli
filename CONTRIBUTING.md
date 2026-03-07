# Contributing

Thanks for helping with bijux-dag.

## Workflow
1. Create a branch.
2. Make focused changes with tests.
3. Ensure formatting and tests pass.
4. Open a PR with a clear summary.

No feature work is accepted without a declared owner, a written contract, and tests.

## Code Style
- Prefer small, readable modules.
- Keep JSON parsing strict.
- Keep scheduling deterministic.
- Use `use ... as _;` only in tests/benches or crate entrypoints when required for dependency-touch accounting; otherwise use explicit imports.
- Do not introduce `todo!`, `unimplemented!`, or placeholder user-facing text in stable surfaces without a policy exception and owner.

## Tests
Run all tests with:
```
cargo test --workspace
```

## Make Targets
- `make test`
- `make lint`
- `make security`

## License
By contributing, you agree your contributions are licensed under Apache-2.0.
