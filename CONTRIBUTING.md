# Contributing to Bijux CLI

This document is intentionally short. It only lists commands and rules that exist in this repository today.

## Prerequisites

- Python 3.11 or newer
- Rust toolchain (stable)
- GNU Make

Optional tools for some Rust targets:

- `cargo-nextest` for `make test-rs` and `make test-all-rs`
- `cargo-deny` and `cargo-audit` for `make audit-rs`
- `cargo-llvm-cov` for `make coverage-rs`

## Setup

```bash
git clone https://github.com/bijux/bijux-cli.git
cd bijux-cli
make install
```

`make install` creates `artifacts/python/.venv` and installs `crates/bijux-cli-python` in editable mode with dev dependencies.

## Commands

Use `make help` to see current targets.

Python and docs:

- `make test-py`
- `make lint-py`
- `make security-py`
- `make build-py`
- `make docs`
- `make docs-serve`

Rust:

- `make fmt-rs`
- `make lint-rs`
- `make test-rs`
- `make test-all-rs`
- `make audit-rs`
- `make coverage-rs`

Other:

- `make all` runs: `clean -> install -> test -> lint -> security -> docs -> build`
- `make test`, `make lint`, `make security`, and `make build` map to Python targets

Direct pytest invocation (without Make):

```bash
pytest -c configs/python/pytest.ini crates/bijux-cli-python/tests/python -q
```

## Pull Requests

Before opening a PR, run the checks relevant to your change.

Typical baseline:

```bash
make test-py
make lint-py
make docs
make fmt-rs
make lint-rs
make test-rs
```

Keep PRs focused and small enough to review.

## Changelog Rules

- Add new notes only under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md).
- Do not rewrite notes for released versions (`0.2.0`, `0.1.3`, etc.).
- If a released section was edited by mistake, restore it and document the correction in `Unreleased`.

## Commit Messages

Conventional commit style is preferred:

```text
<type>(<scope>): <summary>
```

Examples:

- `fix(cli): normalize --version behavior`
- `docs(changelog): clarify unreleased notes`

