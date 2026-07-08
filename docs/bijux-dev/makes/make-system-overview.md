---
title: Make System Overview
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Make System Overview

Use this page when you want the fastest understanding of the repository make
surface: what it is for, why it is split into fragments, and where a target is
likely to live.

The make system is the shell entrypoint for most repository-wide workflows. It
is split into focused fragments so contributors can find ownership quickly
instead of reading one huge root file to understand everything.

## Where Targets Usually Live

| Make fragment | What it owns |
| --- | --- |
| `makes/_macro.mk` | reusable guardrails and helper macros |
| `makes/_internal.mk` | bootstrap, clean, and aggregate repository targets |
| `makes/rust.mk` | Rust quality, testing, coverage, and publication |
| `makes/python.mk` | Python packaging and publication |
| `makes/docs.mk` | MkDocs build, deploy, and documentation checks |
| `makes/gh.mk` | GitHub Actions support and workflow helpers |
| `makes/dag.mk` | DAG governance, evidence, and release-support flows |

## Why The Split Helps

- It keeps root workflows discoverable without flattening unrelated logic.
- It makes ownership clearer when a target fails.
- It gives maintainers one root surface without pretending every workflow is
  the same kind of task.

## Reader Shortcut

If you know the workflow family already, go straight to the relevant fragment
or handbook page. Use this overview when the question is still "which part of
the make system owns this?"

## Continue Reading

- [Root Entrypoints](root-entrypoints.md)
- [Repository Layout](repository-layout.md)
- [Authoring Rules](authoring-rules.md)
