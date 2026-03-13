# Development

## Purpose
This guide provides practical guidance for contributors working on bijux-cli. It exists to reduce accidental violations of core invariants and to help new contributors make changes safely.

## Scope
It covers where to add commands, how to test, and how to avoid breaking core guarantees. It does not cover project governance or release processes.

## Where to Add Commands
Add new CLI commands under `src/bijux_cli/cli/commands/` and keep command handlers thin. Core logic should live in services so it can be tested independently of CLI wiring.

## Where Not to Add Logic
Do not add policy decisions or output routing in infra modules. Those decisions must remain in core so tests can enforce invariants consistently.

## Testing Expectations
Run unit and regression tests for the areas you touched, and confirm that core invariants remain intact. If you change behavior, add or update tests that assert the new guarantees.

## Tooling Notes
The project enforces linting, quality checks, and security scans. These checks are not optional; they preserve the guarantees documented in the concepts section.

Rustdoc remains the primary code documentation path for public Rust APIs. Use
website docs for behavior and workflows, and use Rustdoc plus `bijux dev cli
rustdoc audit` for code-level API truth.

## Build And Release Discipline

Prefer deterministic, repository-local workflows:

- build from a clean checkout
- use pinned toolchains and lockfiles
- avoid relying on mutable host state
- record release artifacts and checksums from the tagged commit

Minimal local build verification:

```bash
cargo build --locked --workspace
cargo test --locked --workspace
```

## Release Review

Before tagging a release, confirm:

1. CI is green for the release commit.
2. Live maintainer checks have been run from a clean checkout.
3. Current parity and known-gap review is complete.
4. Package-health and runtime-identity reports are green.
5. Docs build succeeds.
6. `bijux version` and `bijux cli doctor` pass in a clean environment.

Use live commands first. Generated artifacts are disposable local output:

- `bijux dev cli status --format json --no-pretty`
- `bijux dev cli parity --format json --no-pretty`
- `bijux dev cli docs-audit --format json --no-pretty`
- `cargo test --workspace`
- `python3 -m pytest crates/bijux-cli-python/tests/python`

## Rollback And Compatibility Checks

If a release candidate or published version regresses:

1. stop promoting the new version
2. reinstall the last known-good version through the same channel
3. verify with `bijux version`, `bijux cli paths`, and `bijux cli doctor`
4. pin the rollback version in CI and deployment manifests
5. capture the regression with impact and reproduction details

For maintainer-facing compatibility review, prefer live runtime checks and the
two explicit compatibility comparisons:

```bash
bijux dev cli runtime-identity --json --no-pretty
bijux dev cli parity --json --no-pretty
python3 -m pytest crates/bijux-cli-python/tests/python/test_runtime_parity.py
BIJUX_ENABLE_STABLE_PYPI_PARITY=1 python3 -m pytest -m nightly crates/bijux-cli-python/tests/python/test_stable_release_compatibility.py
```

## References
- [Contributor mental model](contributor-mental-model.md)
- [System overview](../10-architecture/system-overview.md)
