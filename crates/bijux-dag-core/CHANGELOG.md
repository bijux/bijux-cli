# Changelog

All notable changes to **bijux-dag-core** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-19

### Added
- First public crates.io release of `bijux-dag-core` as the deterministic graph
  kernel behind `bijux-dag`.
- Stable and prelude import lanes for supported graph authoring, validation,
  canonicalization, and planning workflows.
- Strict parsing and schema validation for the `bijux-dag/v0.1` graph contract.
- Semantic validation for references, topology, paths, effects, resources,
  retries, cache declarations, and output contracts.
- Deterministic graph canonicalization, topology ordering, and fingerprint
  inputs for reproducible planning.
- Typed graph inputs with defaults, enum and collection constraints, and
  reference materialization before execution.
- Planner lowering that produces execution plans, node identities, and stable
  planner diagnostics without runtime side effects.
- Branch decisions, conditional edges, trigger rules, and selected-path
  analysis as explicit graph semantics.
- Reusable subgraph composition and deterministic dynamic-expansion contracts.
- Opt-in experimental Rust contracts behind `experimental-public-api`, kept
  outside the default stable documentation surface.
