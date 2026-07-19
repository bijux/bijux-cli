---
title: Release Surfaces
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-07
---

# Release Surfaces

Use this page when you need the release-facing make surface in plain terms:
which root targets prepare, validate, and publish repository outputs, and what
they are responsible for.

Release automation uses make as the shell boundary between workflow triggers
and publication commands. That shell layer matters because release work spans
Rust crates, Python packages, GitHub release assets, and DAG bundles.

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

## What Maintainers Usually Need To Decide

| Question | Start here |
| --- | --- |
| is the repository ready for a release candidate? | `make release-validate-rs` or `make gh-release-validate` |
| what publication plan should run? | `make gh-release-plan-github`, `make gh-release-plan-pypi`, or `make gh-release-plan-crates` |
| are credentials and release prerequisites present? | `make gh-release-require-cargo-token` |
| which artifact is actually being built for DAG distribution? | `make build-dag-release-bundle` |
| which publish step pushes the final payload? | `make publish-rs` or `make publish-py` |

## Release Rule

Release planning and release execution should stay separate. The repo should be
able to explain why a publish job ran before it describes how the publish job
executed.

The release validation suite documented in
[Release Validation Suite](../operations/release-validation-suite.md) is the
required gate between candidate selection and publication. It must run against
a clean tree prepared from committed `HEAD`, not against ambient local worktree
state.

## Continue Reading

- [CI Targets](ci-targets.md)
- [gh-workflows](../gh-workflows/index.md)
- [Release Validation Suite](../operations/release-validation-suite.md)
- [Release Operations](../operations/release-operations.md)
