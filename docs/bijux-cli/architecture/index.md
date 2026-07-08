---
title: Architecture
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# CLI Architecture

The architecture section explains how `bijux-cli` is assembled: module
boundaries, dependency flow, execution pipeline, persistence touchpoints,
integration seams, and risk areas.

## Primary Code Anchors

- `crates/bijux-cli/src/lib.rs`
- `crates/bijux-cli/src/bootstrap/`
- `crates/bijux-cli/src/routing/`
- `crates/bijux-cli/src/interface/`
- `crates/bijux-cli/src/features/`
- `crates/bijux-cli/src/infrastructure/`

## Pages In This Section

- [Root CLI Architecture](reference/root-cli-architecture.md)
- [Module Map](module-map.md)
- [Dependency Direction](dependency-direction.md)
- [Execution Model](execution-model.md)
- [State and Persistence](state-and-persistence.md)
- [Integration Seams](integration-seams.md)
- [Error Model](error-model.md)
- [Extensibility Model](extensibility-model.md)
- [Code Navigation](code-navigation.md)
- [Architecture Risks](architecture-risks.md)

## Reading Rule

Start here when the command runtime already makes sense but the internal shape
does not. Move to Interfaces when the next question is about caller-visible
contracts rather than implementation structure.
