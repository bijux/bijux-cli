---
title: Workspace Layout
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Workspace Layout

`bijux-core` uses the repository root as a map of responsibilities. A reader
should be able to look at a top-level path and make a good first guess about
what lives there, who owns it, and whether it is authored source, shared
contract, automation, or generated output.

That matters because this repository ships public products, private support
crates, shared contracts, published docs, and release automation from one
workspace. A flat or blurry root would hide those boundaries instead of making
them easier to review.

## Root Directory Guide

| Root | What readers should expect to find there | Why it exists as a separate root |
| --- | --- | --- |
| `crates/` | Rust crates and their source trees | crate ownership is the primary code boundary in this workspace |
| `contracts/` | shared machine-readable schemas, truth tables, and inventories | several release and docs claims are enforced from one canonical contract root |
| `docs/` | the authored source for the published handbook site | reader-facing explanation stays separate from code and generated output |
| `makes/` | named Make fragments and reusable command entrypoints | root automation remains visible and reviewable instead of drifting into ad hoc shell scripts |
| `.github/workflows/` | hosted CI, docs, and release workflow entrypoints | repository automation is part of the public maintenance story |
| `artifacts/` | generated local or CI output that should not pollute authored roots | generated output gets one predictable landing zone |

## How To Read The Root Quickly

### `crates/` means package ownership

When you need the code that owns a behavior, start here. The workspace is
designed so product and support responsibilities are visible as crate
boundaries instead of being hidden in root utilities.

### `contracts/` means shared truth

When the question is about a schema, release lane, publication boundary, or
another machine-checkable promise, this root usually contains the canonical
asset that docs and tests are expected to follow.

### `docs/` means published explanation

This root is the source of the handbook site. It is for durable reader-facing
content, not a scratch area for temporary notes or generated HTML.

### `makes/` and `.github/workflows/` mean repeatable automation

These roots expose the commands and workflows that contributors, reviewers, and
release maintainers are expected to run or trust. If automation matters, it
should have a named home here.

### `artifacts/` means generated output

This root exists so logs, reports, build output, and generated files from local
work do not leak into product or handbook paths unless the task is explicitly
refreshing a governed destination.

## What The Layout Protects

The root layout is trying to keep several repository mistakes from hardening:

- product code mixed with generated output
- shared contracts scattered through crate-local directories
- workflow logic hidden in one-off scripts
- docs treated as a dumping ground instead of a published handbook source
- new top-level directories created before existing ownership boundaries were
  used well

## A Placement Rule That Scales

Before adding a new path, answer these questions in order:

1. Is this code, shared contract, documentation, automation, or generated
   output?
2. Does one of the existing roots already own that category?
3. If the answer feels like "no," is the real problem weak structure inside an
   existing root rather than the need for a new top-level directory?

Most bad layout decisions come from skipping that third question.

## Reading The Repository With This Model

If you already know the category of the thing you are looking for, the root
layout should narrow your search immediately:

- behavior owner: go to `crates/`
- schema or release truth: go to `contracts/`
- published explanation: go to `docs/`
- root command or CI behavior: go to `makes/` or `.github/workflows/`
- generated output from a run: go to `artifacts/`

## Continue Reading

- [Package Map](package-map.md)
- [Package Boundary](package-boundary.md)
- [Platform Overview](platform-overview.md)
- [Core Architecture](../architecture/workspace-topology.md)
