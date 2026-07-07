---
title: release-validation
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-07
---

# release-validation

`release-validation.yml` runs the canonical release-candidate verification
suite for `bijux-core`.

## Trigger

- `push` on `main`
- every `pull_request`

## Job Shape

- checks out the repository at the candidate commit
- sets up Python `3.11` for the release-tree preparation script
- sets up Rust `1.86.0` with `rustfmt` and `clippy`
- restores the shared Rust cache
- runs `make gh-release-validate`

## Local Mirror

- `make gh-release-validate`
- `make release-validate-rs`

`make gh-release-validate` is the CI entrypoint. It delegates to
`make release-validate-rs`, which executes the exact local release validation
suite from a clean tree prepared from committed `HEAD`.

For the authoritative command inventory, artifact outputs, and release-candidate
failure ownership, use
[Release Validation Suite](../operations/release-validation-suite.md).

## Failure Ownership

- formatter, clippy, test, doc, package, or publish failures belong to the release candidate itself
- workflow setup failures belong to `.github/workflows/release-validation.yml` or `makes/gh.mk`
- clean-tree export failures belong to `.github/scripts/prepare_release_tree.py`

## Next Reads

- [ci](ci.md)
- [Release Validation Suite](../operations/release-validation-suite.md)
- [CI Targets](../makes/ci-targets.md)
