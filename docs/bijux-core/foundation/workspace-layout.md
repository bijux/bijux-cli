---
title: Workspace Layout
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Workspace Layout

The root of `bijux-core` is intentionally opinionated. The directory layout is
part of the repository contract: source, contracts, docs, automation, and
generated outputs do not share a bucket just because they all matter.

That separation is what lets a reader understand the purpose of a path before
opening it and what lets a reviewer tell authored source from generated output
without guesswork.

## Root Layout

| Root | What belongs there |
| --- | --- |
| `crates/` | Rust package ownership boundaries |
| `contracts/` | shared machine-checkable contract assets |
| `docs/` | published handbook sources |
| `makes/` | repository command entrypoints |
| `.github/workflows/` | hosted automation entrypoints |
| `artifacts/` | generated outputs that must stay out of tracked roots |

## How To Read The Root

### `crates/`

This is where Rust package ownership is made explicit. If a behavior belongs to
one executable or library surface, the owning code should be here rather than
in a repository root helper.

### `contracts/`

This is where machine-checkable shared truth lives: schemas, release tables,
and other assets that multiple programs or docs rely on.

### `docs/`

This is the authored source for published reader-facing documentation, not a
dumping ground for generated site output or transient notes.

### `makes/` and `.github/workflows/`

These roots hold repeatable automation entrypoints. If a workflow matters often
enough to document or enforce, it should be visible here rather than hidden in
ad hoc shell history.

### `artifacts/`

This is the default home for generated outputs from local and CI runs unless a
command is explicitly refreshing a governed destination such as checked docs.

## Layout Rule

Root directories should make ownership more obvious, not less. If a new root
directory weakens that rule, it needs repository-handbook justification.

## What This Layout Tries To Prevent

- source code mixed with generated artifacts
- handbook sources mixed with contracts or workflow automation
- new roots that exist only because ownership was not made explicit elsewhere

## Practical Placement Questions

When a new file or directory appears, ask:

1. is this authored source, shared contract, automation, or generated output?
2. does it already have an owned root that matches that purpose?
3. if not, is the problem really missing structure inside an existing root
   rather than a need for a new top-level directory?

Most layout problems come from answering that third question too quickly with
"new root."

## Continue Reading

- [Package Map](package-map.md)
- [Package Boundary](package-boundary.md)
- [Platform Overview](platform-overview.md)
- [Core Architecture](../architecture/workspace-topology.md)
