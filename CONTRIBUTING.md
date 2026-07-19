# Contributing to bijux-core

This document is the operational entrypoint for repository contributors. It
lists the commands, review rules, and evidence expectations enforced by the
current tree.

When this checkout is part of a multi-repository Bijux workspace, read the
workspace-root `AGENTS.md` before contributing.

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

Use `make help` to see current targets. The root entrypoints have deliberately
different scopes:

| Command | Scope |
| --- | --- |
| `make fmt` | Rust formatting verification |
| `make lint` | Rust Clippy across the workspace with warnings denied |
| `make test` | fast Rust release-profile lane plus Python tests marked `not nightly` |
| `make test-slow` | governed slow Rust tests only |
| `make test-all` | all Rust tests, including ignored tests, with retries disabled |
| `make docs-check` | documentation contracts, strict build, navigation, and publication budget |
| `make security` | Rust dependency policy and Python security checks |
| `make build` | Python wheel and source distribution |

Language-specific entrypoints remain available when the change is bounded:

| Surface | Commands |
| --- | --- |
| Rust | `make fmt-rs`, `make lint-rs`, `make test-rs`, `make test-slow-rs`, `make test-all-rs`, `make audit-rs`, `make coverage-rs` |
| Python | `make test-py`, `make test-nightly-py`, `make lint-py`, `make security-py`, `make build-py` |
| Documentation | `make docs`, `make docs-check`, `make docs-serve` |

Direct tool invocation is appropriate for focused diagnosis. It proves only
the selected surface. For Python:

```bash
artifacts/python/.venv/bin/pytest \
  -c configs/python/pytest.ini \
  crates/bijux-cli-python/tests/python \
  -q
```

For focused Rust runtime and command-surface checks:

```bash
cargo nextest run -p bijux-cli -E 'test(routing::)'
cargo run -q -p bijux-dev --bin bijux-dev-cli -- status --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- parity --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- docs-audit --format json --no-pretty
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
```

For focused DAG behavior:

```bash
cargo nextest run -p bijux-dag-app -E 'test(cli_contract)'
cargo run -q -p bijux-dag-cli --bin bijux-dag -- --help
cargo run -q -p bijux-dev --bin bijux-dev-dag -- verify evidence-release-set
```

If your change touches release identity, installation guidance, or version
documentation, also verify the built runtime directly:

```bash
cargo build -p bijux-cli --bin bijux
./artifacts/rust/target/debug/bijux version --format json --no-pretty
```

Cargo output remains under `artifacts/rust/target/` through
`.cargo/config.toml`. Python environments, caches, coverage, and build products
remain under `artifacts/python/`.

## Pull Requests

Before opening a PR, run the checks relevant to the changed boundary.

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

`make all` is not the same as `make test-all`: it combines the repository's
configured format, lint, security, default test, and build entrypoints, while
`make test-all` is the complete Rust test lane. Report the exact command rather
than saying only that "all tests" passed.

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

## Documentation Structure

Each documentation surface has one job:

| Surface | Authority |
| --- | --- |
| root `README.md` | repository products, installation, package families, development entrypoints, and documentation routing |
| crate `README.md` | package purpose, public boundary, use, and crate-local documentation index |
| `docs/bijux-*` | public handbooks rendered by MkDocs |
| `crates/*/docs/` | internal package architecture, contracts, development, operations, and verification |
| `docs/spec/` | canonical or generated cross-package technical contracts |
| `docs/reports/` | checked-in reproducible observations with named producers |

Public handbook roots contain only `index.md` and named directories. Place a
page under the directory that owns its subject rather than accumulating loose
root pages. Crate-local documentation stays flat and contains no more than ten
Markdown pages per crate; a new page must own a distinct package concern that
cannot be explained clearly in an existing page.

Use Mermaid when a relationship, state transition, dependency direction, or
execution sequence is clearer as a diagram. Introduce the question the diagram
answers and explain the important boundary afterward. Do not add decorative
flowcharts that merely repeat a nearby list.

For documentation changes, run:

```bash
make docs-governance-lint
cargo test --locked -p bijux-dev --test docs_source_reference_contracts
make docs-check
```

`make docs-check` is the publication gate. If a managed standards mismatch
blocks it, report that blocker and repair the synchronized source through its
owner; do not edit generated shared content downstream to manufacture a pass.

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
- `refactor(runtime): isolate scheduler policy from backend adapters`

Scopes and subjects describe durable ownership and intent. Do not use
sequence labels, delivery-stage names, placeholder terminology, or generic
subjects that only make sense in the context of one work session.
