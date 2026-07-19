---
title: PyPI Release
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
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
`make gh-release-plan-pypi`, and the release Rust toolchain is `1.85.0`.

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
5. publishes the prebuilt files through `make publish-py PUBLISH_BUILD=0`.

The workflow supplies `PYPI_API_TOKEN` to the repository publish command.
`make publish-py` verifies the distributions with Twine and uses
`--skip-existing`, so a retry does not replace an existing file. The reusable
workflow also supports an artifact mode with trusted publishing, but that is
not the mode selected by this repository's release configuration.

## Failure Meaning

A CI wait failure means the tagged commit has not passed the required gate. A
planning failure means tag or registry state could not be established. Build,
Twine, or upload failures belong to the Python distribution lane.

PyPI publication is not atomic with crates.io, GHCR, or the GitHub Release.
Those jobs run independently from the same tag. After a partial release, keep
the tag fixed, inspect registry state, and rerun only the failed lane; do not
retag a different commit with the same version.

## Source Authorities

- `.github/workflows/release-on-tag.yml`
- `.github/workflows/release-pypi.yml`
- `.github/release.env`
- `makes/gh.mk`
- `makes/python.mk`

## Next Reads

- [Rust Crates Release](release-crates.md)
- [GitHub Release](release-github.md)
- [Release Operations](../operations/release-operations.md)
