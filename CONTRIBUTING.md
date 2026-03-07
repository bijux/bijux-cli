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

Fast local feedback:
```bash
make test
```

Full repository verification (includes battle and release evidence gates):
```bash
make test-all
```

`make test` intentionally skips slower and release-oriented suites. Use `make test-all`
before opening a PR when command surfaces, replay/diff behavior, or evidence wiring changes.

## Make Targets
- `make test`
- `make test-all`
- `make lint`
- `make security`

## License
By contributing, you agree your contributions are licensed under Apache-2.0.
