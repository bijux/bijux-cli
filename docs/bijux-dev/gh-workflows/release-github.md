---
title: GitHub Release
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# GitHub Release

`release-github.yml` owns the GitHub Release record and its attached files. It
does not publish crates, PyPI distributions, or GHCR packages. Those are
separate workflows called in parallel by `release-on-tag.yml`.

The workflow is reusable and manually dispatchable. A stable `v*` tag enters
through the tag orchestrator; a manual dispatch must resolve to an enabled,
non-empty build matrix or it fails as a no-op configuration.

## Current Release Shape

`.github/release.env` is the repository-specific configuration authority. It
currently selects:

- `make gh-release-plan-github` for the publication decision;
- Rust `1.85.0` for build preparation;
- one `bijux-cli` build-matrix entry;
- `make build-py` as that entry's build target;
- `artifacts/python/build` as its distribution directory.

The generated `release-artifacts.yml` workflow builds and uploads the matrix
entry as a workflow artifact. The release job downloads matching `*-release`
artifacts and attaches their files to the GitHub Release. The present matrix
does not build or attach a `bijux-dag` binary bundle; that bundle exists as a
local Make surface but is not selected by this workflow configuration.

## Publication Path

For an eligible release, the workflow:

1. builds every configured matrix entry without fail-fast cancellation;
2. checks out the exact candidate commit and waits for `ci.yml` on tag pushes;
3. verifies that the commit carries a stable release tag;
4. downloads the build artifacts from the current workflow run;
5. creates or updates the GitHub Release for that tag.

Generated release notes are enabled. Existing attached file names are
overwritten, while the release record itself is not deleted unless
`delete_existing_release` is explicitly enabled. Missing file matches are
tolerated by the current configuration, so maintainers must inspect the
release's attached assets rather than treating record creation alone as proof
that every intended artifact exists.

## Failure Meaning

Build failure blocks the release job because it depends on the complete matrix.
A CI wait or planning failure means the candidate is not publishable. An asset
download or release API failure belongs to the GitHub lane.

This workflow is not a transaction coordinator. A GitHub Release can fail
after another registry lane succeeds, or succeed while another lane fails.
Keep the tag immutable, inspect each channel independently, and rerun only the
failed workflow.

## Source Authorities

- `.github/workflows/release-on-tag.yml`
- `.github/workflows/release-github.yml`
- `.github/workflows/release-artifacts.yml`
- `.github/release.env`
- `makes/gh.mk`
- `makes/python.mk`

## Next Reads

- [Rust Crates Release](release-crates.md)
- [PyPI Release](release-pypi.md)
- [Release Surfaces](../makes/release-surfaces.md)
