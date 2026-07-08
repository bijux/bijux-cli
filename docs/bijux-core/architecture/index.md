---
title: Core Architecture
audience: mixed
type: section-index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Core Architecture

This section covers the repository-level architecture that coordinates CLI,
DAG, the Python bridge, and the maintainer surface.

Use [Foundation](../foundation/index.md) first when the repository split itself
is still unclear. Use this section when the ownership model is already clear
and the remaining question is structural.

## Architecture Priorities

- one workspace authority at the repository root
- explicit crate ownership with one-way dependency rules
- stable command and artifact behavior across CLI and DAG programs
- maintainers operate through dedicated control-plane paths

## Related Root Pages

- [Foundation](../foundation/index.md)
- [Operations](../operations/index.md)
- [Repository Handbook](../index.md)

## Pages In This Section

- [System Overview](system-overview.md)
- [Workspace Topology](workspace-topology.md)
- [Dependency Direction](dependency-direction.md)
- [Runtime Surfaces](runtime-surfaces.md)
- [State and Configuration](state-and-configuration.md)
- [Distribution Model](distribution-model.md)
- [Maintainer Control Plane](maintainer-control-plane.md)
- [Artifact and Contract Flow](artifact-and-contract-flow.md)
- [Architecture Risks](architecture-risks.md)

## Reading Rule

Start with System Overview when you need the whole workspace shape, then move to
the narrower pages once the main split is already clear.
