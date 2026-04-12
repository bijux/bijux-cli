---
title: release-crates
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# release-crates

`release-crates.yml` publishes the Rust release surface after a version tag is
present and CI on the tagged commit is healthy.

## Trigger

- `push` on tags matching `v*`
- manual `workflow_dispatch`

## Job Shape

- wait for `ci.yml` to pass on the tagged commit
- decide whether crates publication is needed with `make gh-release-plan-crates`
- verify crates.io credentials
- publish through `make publish-rs`

## Next Reads

- [release-pypi](release-pypi.md)
- [release-github](release-github.md)
- [Release Surfaces](../makes/release-surfaces.md)
