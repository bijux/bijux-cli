---
title: PyPI Release
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-24
---

# PyPI Release

`release-pypi.yml` owns publication of the `bijux-cli` Python distribution.
It is a reusable workflow with a manual-dispatch entrypoint. Tag pushes do not
trigger it directly: `release-on-tag.yml` calls it alongside the crates.io,
GHCR, and GitHub Release workflows.

## Eligibility

The resolve job combines workflow inputs, `.github/release.env`, repository
variables, and workflow defaults. For this repository the configured mode is
`maturin`, publication planning is delegated to
`make gh-release-plan-pypi`, and the release Rust toolchain is `1.86.0`.

The plan accepts only a stable `vMAJOR.MINOR.PATCH` tag that points at the
candidate commit. It probes `bijux-cli` on PyPI and returns a no-op when that
version already exists. A manual dispatch must explicitly resolve to an
enabled run; a disabled manual run fails rather than appearing to publish.

## Publication Path

For an eligible Maturin release, the workflow:

1. checks out the exact candidate commit with full tag history;
2. waits for `ci.yml` on that commit when invoked by a tag push;
3. prepares a clean release tree stamped with the tag version;
4. builds a manylinux wheel and source distribution from
   `crates/bijux-cli-python/Cargo.toml`;
5. publishes the prebuilt files from `artifacts/python/build` through PyPI
   trusted publishing.

For this repository, the managed `maturin` lane now uses the same trusted
publisher action as the shared artifact mode. If trusted publishing fails only
because the package does not exist yet, the workflow can bootstrap with
`PYPI_API_TOKEN` when that secret is configured. A repository can still set
`BIJUX_PYPI_PUBLISH_COMMAND` to replace the managed publisher intentionally,
but `bijux-core` does not set that override.

## Failure Meaning

A CI wait failure means the tagged commit has not passed the required gate. A
planning failure means tag or registry state could not be established. Build,
trusted-publisher, or token-bootstrap failures belong to the Python
distribution lane.

PyPI publication is not atomic with crates.io, GHCR, or the GitHub Release.
Those jobs run independently from the same tag. After a partial release, keep
the tag fixed, inspect registry state, and rerun only the failed lane; do not
retag a different commit with the same version.

## Source Authorities

- `.github/workflows/release-on-tag.yml`
- `.github/workflows/release-pypi.yml`
- `.github/release.env`
- `makes/gh.mk`

## Next Reads

- [Rust Crates Release](release-crates.md)
- [GitHub Release](release-github.md)
- [Release Operations](../operations/release-operations.md)
