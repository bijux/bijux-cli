---
title: release-crates
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-04
---

# release-crates

`release-crates.yml` publishes the Rust release surface after a version tag is
present and CI on the tagged commit is healthy.

## Trigger

- `push` on tags matching `v*`
- manual `workflow_dispatch`

## Job Shape

- wait for `ci.yml` to pass on the tagged commit
- provision Rust `1.86.0` so crates publication uses the same toolchain as the workspace and CI
- decide whether crates publication is needed with `make gh-release-plan-crates`
- verify crates.io credentials
- publish through `make publish-rs`

## Publication Order

The default publish order is dependency-first:

- `bijux-dag-core`
- `bijux-dag-artifacts`
- `bijux-dag-runtime`
- `bijux-dag-app`
- `bijux-dag-cli`
- `bijux-dag-testkit`
- `bijux-cli`

That order keeps the DAG crate family coherent on crates.io before the separate
`bijux` runtime crate is published.

## Next Reads

- [release-pypi](release-pypi.md)
- [release-github](release-github.md)
- [Release Surfaces](../makes/release-surfaces.md)
