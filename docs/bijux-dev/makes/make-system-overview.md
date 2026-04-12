---
title: Make System Overview
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Make System Overview

`Makefile` includes `makes/root.mk`, which composes the repository make surface
from focused fragments instead of leaving behavior in one large file.

## Composition

- `makes/_macro.mk` for reusable guardrails
- `makes/_internal.mk` for bootstrap, clean, and aggregate quality targets
- `makes/rust.mk` for Rust quality, coverage, and publication
- `makes/python.mk` for Python packaging and publication
- `makes/docs.mk` for MkDocs build, deploy, and structure checks
- `makes/gh.mk` for GitHub Actions support targets
- `makes/dag.mk` for DAG governance and evidence flows

## Next Reads

- [Root Entrypoints](root-entrypoints.md)
- [Repository Layout](repository-layout.md)
- [Authoring Rules](authoring-rules.md)
