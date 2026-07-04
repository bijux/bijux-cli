---
title: release-github
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-04
---

# release-github

`release-github.yml` builds the public release bundles, publishes the matching
GHCR archives, and creates the GitHub release entry.

## Trigger

- `push` on tags matching `v*`
- manual `workflow_dispatch`

## Job Shape

- wait for `ci.yml` to pass on the tagged commit
- decide publication with `make gh-release-plan-github`
- prepare the stamped release tree
- build the `bijux-cli` Python wheel and source distribution
- build the stamped `bijux-dag` binary tarball through `make build-dag-release-bundle`
- generate checksums and release notes
- publish both release families to GHCR and create the GitHub release

## Release Assets

- `bijux-cli` contributes the Python distribution artifacts used by the GitHub
  release and GHCR publication lanes.
- `bijux-dag` contributes a stamped `.tar.gz` archive containing the
  `bijux-dag` executable, bundle metadata, install notes, and checksums.

## Next Reads

- [release-crates](release-crates.md)
- [release-pypi](release-pypi.md)
- [Release Surfaces](../makes/release-surfaces.md)
