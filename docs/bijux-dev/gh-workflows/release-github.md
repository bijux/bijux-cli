---
title: release-github
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# release-github

`release-github.yml` builds the GitHub release bundle, publishes the GHCR
archive, and creates the GitHub release entry.

## Trigger

- `push` on tags matching `v*`
- manual `workflow_dispatch`

## Job Shape

- wait for `ci.yml` to pass on the tagged commit
- decide publication with `make gh-release-plan-github`
- prepare the stamped release tree
- build the Python wheel and source distribution
- generate checksums and release notes
- publish the bundle to GHCR and create the GitHub release

## Next Reads

- [release-crates](release-crates.md)
- [release-pypi](release-pypi.md)
- [Release Surfaces](../makes/release-surfaces.md)
