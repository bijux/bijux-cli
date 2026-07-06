---
title: Modeled and Future Surfaces
audience: mixed
type: foundation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Modeled and Future Surfaces

This page separates implemented DAG capabilities from modeled, simulated, or
future-looking surfaces that should not be presented as release-ready runtime
behavior.

## Modeled Surfaces

- simulated platform namespaces behind explicit opt-in
- modeled distributed and federated execution boundaries
- modeled remote worker execution
- modeled SLURM execution through the shared runtime lane
- internal release, performance, and governance routes that exist for
  repository verification rather than public operator compatibility

## Future Surfaces

- public enterprise and federation APIs
- Kubernetes execution beyond the current documented contract
- SLURM execution claims beyond the current documented simulated backend

## Documentation Rule

Pages that mention these surfaces must describe them as modeled, hidden,
internal, or future-facing unless the implementation, tests, and release
boundary documents have been promoted together.

When a page mentions performance-related maintainer routes, it should point
readers to `bijux-dev-dag performance-evidence-report` and
`evidence/perf/metadata.json` rather than implying standalone product
capabilities.

## Next Reads

- [Release Boundary](release-boundary.md)
- [Scope and Non-Goals](scope-and-non-goals.md)
- [Known Limitations](../quality/known-limitations.md)
