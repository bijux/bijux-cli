---
title: Release Surfaces
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-04
---

# Release Surfaces

Release automation uses make as the shell boundary between workflow triggers
and publication commands.

## Important Targets

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

## Next Reads

- [CI Targets](ci-targets.md)
- [gh-workflows](../gh-workflows/index.md)
- [Release Operations](../operations/release-operations.md)
