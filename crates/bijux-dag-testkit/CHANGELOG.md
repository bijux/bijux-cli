# Changelog

All notable changes to **bijux-dag-testkit** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-24

### Added
- Declared `bijux-dag-testkit` as the repository-internal support crate for
  shared DAG tests.
- Deterministic graph builders for chains, diamonds, fan-out, disconnected,
  retry, timeout, cache, replay, branch, and failure scenarios.
- Canonical node and edge builders that keep repeated test graphs aligned
  across package boundaries.
- Fake adapter harnesses for exercising runtime contracts without production
  process or service dependencies.
- Product scenario helpers for cross-crate graph, runtime, artifact, and
  application tests.
- Retained manifest normalization and comparison helpers for deterministic
  assertions.
- Evidence-registry readers and asset resolution by governed identifier.
- Compatibility path resolution for repository evidence consumed by existing
  test suites.
- Checked APIs that report missing or malformed evidence rather than relying
  only on panic-based fixture access.
- An explicit internal-only boundary: production crates do not depend on the
  testkit at runtime and the crate is not part of the public crates.io release.
