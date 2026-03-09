# Reproducible Releases

Release generation must be deterministic from a tagged commit.

## Required Inputs

- Semantic version tag (for example `v0.4.2`)
- CI artifacts generated from the same commit SHA
- Locked dependency graphs

## Required Outputs

- Dist artifacts
- Checksums file
- Artifact manifest
- Release tarball

## Validation

Rebuild from the same tag and compare checksums for all emitted artifacts.

