---
title: Rust Crates Release
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Rust Crates Release

`release-crates.yml` owns crates.io publication for the public Rust package
surface. It is reusable and manually dispatchable. Stable tag pushes enter
through `release-on-tag.yml`, which calls this lane independently of PyPI,
GHCR, and GitHub Release publication.

## Eligibility

The workflow resolves its enabled state and commands from inputs,
`.github/release.env`, repository variables, and defaults. The repository
configuration selects Rust `1.85.0`, `make gh-release-plan-crates`, and
`make publish-rs` with real publication and existing-version skips enabled.

`make gh-release-plan-crates` requires a stable `vMAJOR.MINOR.PATCH` tag on the
candidate commit. It probes every public package at that version and returns
only packages absent from crates.io. If all packages exist, the lane ends
without publishing.

## Package Boundary

The canonical public/private classification lives in
`contracts/foundation/workspace_package_boundary.v1.json`. Publication follows
dependency order:

1. `bijux-dag-core`
2. `bijux-dag-artifacts`
3. `bijux-dag-runtime`
4. `bijux-dag-app`
5. `bijux-dag-cli`
6. `bijux-cli`

`bijux-dag-testkit`, `bijux-dev`, and `bijux-cli-python` are repository support
packages and are not published as Rust crates.

## Publication Path

For an eligible run, the workflow waits for `ci.yml` on a tag push, resolves
the unpublished package set, verifies `CARGO_REGISTRY_TOKEN`, and passes the
tag-derived version and package list to `make publish-rs`.

The Make target creates a clean release tree stamped with the release version,
resolves each package version from Cargo metadata, and invokes
`cargo publish --locked` in dependency order. It checks crates.io before each
upload and also recognizes the registry's already-uploaded response. These two
guards make retries safe; they do not make the multi-package release atomic.

## Failure Meaning

A failure before publication leaves registry state unchanged. A failure after
one or more uploads leaves a partial crates.io release because registries do
not support rollback. Keep the tag fixed, confirm which packages exist, and
rerun the lane. The planner and publisher will skip completed packages and
continue in dependency order.

## Source Authorities

- `.github/workflows/release-on-tag.yml`
- `.github/workflows/release-crates.yml`
- `.github/release.env`
- `makes/gh.mk`
- `makes/rust.mk`
- `contracts/foundation/workspace_package_boundary.v1.json`

## Next Reads

- [PyPI Release](release-pypi.md)
- [GitHub Release](release-github.md)
- [Release Surfaces](../makes/release-surfaces.md)
