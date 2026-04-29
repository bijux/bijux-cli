---
title: CLI Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# CLI Surface

This page explains how the CLI is organized when you approach it as a command
surface rather than as source code.

The important idea is simple: many commands exist, but they still collapse into
a small set of route groups with one shared flag model.

## Route Map

```mermaid
flowchart LR
    root["bijux"] --> runtime["runtime checks"]
    root --> state["state management"]
    root --> plugins["plugin lifecycle"]
    root --> interaction["interactive and help routes"]
```

## Stable Route Groups

- runtime probes: `status`, `audit`, `docs`, `doctor`, `version`
- state management: `config`, `history`, `memory`
- plugin lifecycle: `plugins` and `cli plugins ...`
- interaction: `repl`, `completion`, `help`

## Global Flag Contract

- `--format|-f`: `text`, `json`, `yaml`
- `--json` and `--text`: explicit format aliases
- `--pretty` and `--no-pretty`: render style toggles
- `--color`: ANSI policy
- `--log-level`: telemetry and diagnostics verbosity
- `--quiet`: suppress output streams after successful execution

## Code Anchors

- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/routing/model.rs`
- `crates/bijux-cli/src/interface/cli/help.rs`
- `crates/bijux-cli/tests/data/golden/cli_surface/`

## CLI Surface Rules

- aliases should normalize to canonical paths before route execution
- help output should match parser model and route catalog
- command additions require docs update and route test coverage
- root and `cli` prefixed forms must remain coherent

## Reading Rule

Use this page when the question is which command family owns a behavior before
you narrow the search to one concrete route.

## Next Reads

- [Operator Workflows](operator-workflows.md)
- [Entrypoints and Examples](entrypoints-and-examples.md)
- [Compatibility Commitments](compatibility-commitments.md)
