---
title: Current Implemented Capabilities
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-06
---

# Current Implemented Capabilities

This page records what the repository can honestly claim today across the CLI,
DAG runtime, artifact, and maintainer surfaces.

## Repository Capability Boundary

- `bijux-cli` provides the public `bijux` command runtime, route normalization,
  plugins, and structured output
- `bijux-dag` provides local DAG validation, planning, execution, replay,
  evidence inspection, and cache-aware reruns
- `bijux-dev` provides repository governance, evidence reports, and release
  verification for maintainers

## Claim Discipline

- capability claims must map to checked-in docs, code, contracts, or tests
- modeled or future-facing surfaces must be called out separately
- handbook pages should prefer implemented scope over aspirational scope

## Next Reads

- [Documentation System](documentation-system.md)
- [Package Boundary](package-boundary.md)
- [Modeled and Future Surfaces](../../bijux-dag/foundation/modeled-and-future-surfaces.md)
