---
title: Workspace Layout
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# Workspace Layout

Use this page when the repository root feels busy and you need the quick answer
to a practical question: where does a thing belong?

The layout is meant to make ownership obvious at a glance. Source code,
contracts, docs, workflows, and generated outputs live in separate roots so a
reader can tell what a directory is for before opening it.

## Root Layout

| Root | What belongs there |
| --- | --- |
| `crates/` | Rust package ownership boundaries |
| `contracts/` | shared machine-checkable contract assets |
| `docs/` | published handbook sources |
| `makes/` | repository command entrypoints |
| `.github/workflows/` | hosted automation entrypoints |
| `artifacts/` | generated outputs that must stay out of tracked roots |

## Layout Rule

Root directories should make ownership more obvious, not less. If a new root
directory weakens that rule, it needs repository-handbook justification.

## What This Layout Tries To Prevent

- source code mixed with generated artifacts
- handbook sources mixed with contracts or workflow automation
- new roots that exist only because ownership was not made explicit elsewhere

## Continue Reading

- [Package Map](package-map.md)
- [Package Boundary](package-boundary.md)
- [Core Architecture](../architecture/workspace-topology.md)
