# Contributing to bijux-core

This document stays intentionally operational. It lists the commands, review
rules, and evidence expectations that govern this repository today.

## Prerequisites

- Python 3.11 or newer
- Rust toolchain (stable, managed by `rust-toolchain.toml`)
- GNU Make

Optional tools for some Rust targets:

- `cargo-nextest` for `make test-rs` and `make test-all-rs`
- `cargo-deny` and `cargo-audit` for `make audit-rs`
- `cargo-llvm-cov` for `make coverage-rs`

## Setup

```bash
git clone https://github.com/bijux/bijux-core.git
cd bijux-core
make bootstrap
```

`make bootstrap` prepares the repository-managed virtualenv under `artifacts/`
and installs `crates/bijux-cli-python` in editable mode with dev dependencies.

Workspace package manifests stay on the current development line. Tagged release
publishes stamp the exact tag version into a temporary release tree instead of
committing release-only version edits back into the working branch. Untagged
checkout builds still derive runtime identity from the latest real `v*` tag in
Git, so the source tree can move ahead for the next release without claiming
that newer release from `bijux version`.

## Commands

Use `make help` to see current targets.

Python and docs:

- `make test-py`
- `make lint-py`
- `make security-py`
- `make build-py`
- `make docs`
- `make docs-check`
- `make docs-serve`

Rust:

- `make fmt-rs`
- `make lint-rs`
- `make test-rs`
- `make test-all-rs`
- `make audit-rs`
- `make coverage-rs`

Other:

- `make fmt` runs `fmt-rs` and `fmt-py`
- `make lint` runs `lint-rs` and `lint-py`
- `make test` runs `test-rs` and `test-py`
- `make security` runs `audit-rs` and `security-py`
- `make build` runs `build-py`

Direct pytest invocation (without Make):

```bash
pytest -c configs/python/pytest.ini crates/bijux-cli-python/tests/python -q
```

Direct Rust runtime verification that is useful for command-surface and docs
changes:

```bash
cargo test -p bijux-cli
cargo run -q -p bijux-dev --bin bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- parity --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- docs-audit --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
```

Direct DAG verification that is useful for graph/runtime behavior and DAG docs
changes:

```bash
cargo test -p bijux-dag-app
cargo run -q -p bijux-dag-cli --bin bijux-dag -- dag --help
cargo run -q -p bijux-dev --bin bijux-dev-dag -- verify evidence-release-set
```

If your change touches release identity, installation guidance, or version
documentation, also verify the built runtime directly:

```bash
cargo build -p bijux-cli --bin bijux
./artifacts/rust/target/debug/bijux version --format json --no-pretty
```

## Pull Requests

Before opening a PR, run the checks relevant to your change.

Typical baseline:

```bash
make fmt
make lint
make test
make docs-check
```

For release-facing changes, prefer this fuller pass before asking for review:

```bash
make all
```

Keep PRs focused and small enough to review.

Runtime and maintainer surfaces must stay separate:

- `bijux` documents only the runtime namespace shipped to end users
- `bijux-dev-cli` remains the workspace maintainer control plane
- install aliases such as `bijux install dev-cli` resolve package names; they do not make maintainer commands part of the runtime command surface

## Docs Honesty Rules

Public docs are part of the product contract. When code changes behavior:

- update the user-facing docs in the same change
- remove claims that are no longer true instead of explaining them away
- prefer command output, tests, or maintainer reports over hand-wavy prose
- treat README and contract pages as higher-scrutiny surfaces than architecture notes

If a command is still partial, internal, workspace-only, or unsupported, say
that directly.

When describing release posture, use the latest real git tag or published
artifact as the release source of truth instead of the bumped workspace manifest
line by itself.

## Evidence Rules

Before merging a behavior claim about the runtime, make sure at least one of
these is true:

- the relevant tests pass locally
- the behavior is visible from the current binary output
- the maintainer reports agree with the docs you changed

Do not leave stale docs in place because the code changed "recently". That is
exactly the kind of drift this repository tries to avoid.

## Changelog Rules

- Add new notes only under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md).
- Do not rewrite notes for released versions (`0.2.0`, `0.1.3`, etc.).
- If a released section was edited by mistake, restore it and document the correction in `Unreleased`.

## Commit Messages

Use conventional commit style:

```text
<type>(<scope>): <summary>
```

Examples:

- `fix(cli): normalize --version behavior`
- `docs(changelog): clarify unreleased notes`
- `refactor(runtime): replace placeholder command handlers`
