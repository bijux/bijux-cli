---
title: makes
audience: mixed
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Dev Make System

The `makes/` section explains the shared command surface that ties local work,
CI validation, docs publication, DAG governance, and release automation
together.

Use it when the question is about which root make target owns a workflow, how
targets are grouped, or where repository-wide commands are defined.

## Pages In This Section

- [Make System Overview](make-system-overview.md)
- [Root Entrypoints](root-entrypoints.md)
- [Environment Model](environment-model.md)
- [Repository Layout](repository-layout.md)
- [Package Dispatch](package-dispatch.md)
- [CI Targets](ci-targets.md)
- [Package Contracts](package-contracts.md)
- [Release Surfaces](release-surfaces.md)
- [Authoring Rules](authoring-rules.md)

## Reading Rule

Start here when the root command surface is still unclear. Move to GitHub
workflows when the next question is about hosted automation rather than local
or repository-level command entrypoints.
