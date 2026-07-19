# Changelog

All notable changes to **bijux-dag-artifacts** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-20

### Added
- First public crates.io release of `bijux-dag-artifacts` as the retained
  evidence layer for `bijux-dag`.
- Stable and prelude import lanes for supported artifact read, write, and
  verification workflows.
- Typed run manifests, node traces, output indexes, and storage records for
  finalized run directories.
- Deterministic path and platform helpers for run, cache, node, and artifact
  layouts.
- Filesystem-backed stores with bounded read and write services for retained
  evidence.
- SHA-256 hashing, output indexing, proof records, and schema validation for
  artifact integrity.
- Run-layout verification that refuses incomplete, inconsistent, or
  untrusted evidence rather than treating presence as proof.
- Retention policy models for deciding which run and artifact records remain
  available.
- Promotion records and lineage helpers that retain producer and source-run
  identity across artifact lifecycle operations.
- Opt-in experimental contract helpers behind `experimental-public-api`, kept
  outside the default stable documentation surface.
