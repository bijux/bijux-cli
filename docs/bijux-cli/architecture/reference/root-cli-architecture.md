---
title: Root CLI Architecture
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Root CLI Architecture

`bijux` is intentionally a root router, not a monolithic command implementation.
The root process owns only the parts that must stay uniform across every product
and plugin surface.

## Root Responsibilities

- parse global flags and normalize aliases
- resolve built-in, official-app, and plugin namespaces
- enforce output envelopes and exit-code mapping
- delegate product-specific execution without rewriting payload semantics
- keep root help, suggestions, and diagnostics consistent

## Runtime Layers

1. `src/bootstrap/`: process entrypoint and stream wiring
2. `src/routing/`: grammar, normalization, registry, and suggestions
3. `src/interface/cli/`: root command handlers and help rendering
4. `src/kernel/`: execution pipeline and policy application
5. `src/features/`: domain implementations for config, plugins, install, and diagnostics

## Delegation Rules

- built-in root commands stay inside `bijux-cli`
- official apps route through mount descriptors
- Python and Rust mounted apps must still emit root-compatible JSON envelopes
- plugins stay isolated behind manifest and runtime checks

## Review Questions

- does the change belong at the root, or inside an app/plugin surface?
- does it preserve alias normalization and route determinism?
- does it keep stdout/stderr and exit-code behavior stable?
- does it avoid leaking product-specific behavior into the root parser?
