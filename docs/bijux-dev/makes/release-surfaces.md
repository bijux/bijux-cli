---
title: Release Surfaces
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-07
---

# Release Surfaces

Release automation uses make as the shell boundary between workflow triggers
and publication commands.

## Important Targets

- `make release-validate-rs`
- `make gh-release-validate`
- `make gh-release-plan-github`
- `make gh-release-plan-pypi`
- `make gh-release-plan-crates`
- `make gh-release-require-cargo-token`
- `make build-dag-release-bundle`
- `make publish-rs`
- `make publish-py`

## Release Families

- `bijux-cli` builds the Python release artifacts used by PyPI, GitHub
  Releases, and GHCR.
- `bijux-dag` builds a stamped Rust binary bundle under
  `artifacts/rust/build/` for GitHub Releases and GHCR.

## Release Rule

Release planning and release execution should stay separate. The repo should be
able to explain why a publish job ran before it describes how the publish job
executed.

The release validation suite documented in
[Release Validation Suite](../operations/release-validation-suite.md) is the
required gate between candidate selection and publication. It must run against
a clean tree prepared from committed `HEAD`, not against ambient local worktree
state.

## Next Reads

- [CI Targets](ci-targets.md)
- [gh-workflows](../gh-workflows/index.md)
- [Release Validation Suite](../operations/release-validation-suite.md)
- [Release Operations](../operations/release-operations.md)
