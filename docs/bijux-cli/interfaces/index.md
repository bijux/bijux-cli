---
title: Interfaces
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# CLI Interfaces

Use this section when you need to know what external callers can rely on from
`bijux`: command behavior, public Rust imports, generated configuration
reference, typed payloads, and compatibility commitments.

Start here when you are integrating with the CLI, writing automation around
its output, embedding the runtime from Rust, or validating whether a workflow
belongs to the supported public surface.

## Start With The Question You Have

| If you need to... | Open this page |
| --- | --- |
| see the visible `bijux` command contract | [CLI Surface](cli-surface.md) |
| embed or call the public Rust API | [API Surface](api-surface.md) |
| understand config files, scopes, and generated docs | [Configuration Surface](configuration-surface.md) |
| inspect schemas and payload shapes | [Data Contracts](data-contracts.md) |
| check examples before automating against the CLI | [Entrypoints and Examples](entrypoints-and-examples.md) |
| understand what remains stable across releases | [Compatibility Commitments](compatibility-commitments.md) |

## What This Section Covers

- the visible command-line routes, flags, and output expectations
- the stable Rust-facing imports under `src/api`
- generated and human-authored configuration references
- typed command envelopes, plugin manifests, and other contract payloads
- compatibility boundaries for scripts, wrappers, and integrations

## Public Surface In One View

`bijux` exposes the same runtime semantics through more than one entrypoint:

- the installed `bijux` executable
- the Rust crate surface in `bijux-cli`
- the Python-distributed launcher covered by `bijux-cli-python`

This section documents the caller-facing contract shared across those paths.
When a page crosses into Python packaging or bridge mechanics, it points back
to the package documentation instead of redefining the runtime story here.

## Code Anchors

- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/interface/cli/handlers/`
- `crates/bijux-cli/src/api/`
- `crates/bijux-cli/src/contracts/`
- `crates/bijux-cli/tests/routing/`

## Pages In This Section

- [CLI Surface](cli-surface.md)
- [API Surface](api-surface.md)
- [App Integration Guide](app-integration/guide.md)
- [App Integration Scenario](app-integration/scenario.md)
- [Configuration Surface](configuration-surface.md)
- [Config Guide](config/guide.md)
- [Generated Config Reference](config/generated-reference.md)
- [Data Contracts](data-contracts.md)
- [Artifact Contracts](artifact-contracts.md)
- [Entrypoints and Examples](entrypoints-and-examples.md)
- [Examples](examples/command-examples.md)
- [Operator Workflows](operator-workflows.md)
- [Public Imports](public-imports.md)
- [Compatibility Commitments](compatibility-commitments.md)

## Before You Move Deeper

- Stay in this section when the question is about what a caller may depend on.
- Move to Architecture when the next question is how the runtime is assembled
  internally.
- Move to Operations when the question becomes installation, diagnostics,
  release handling, or day-to-day operating practice.
