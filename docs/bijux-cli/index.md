---
title: CLI Handbook
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-07
---

# CLI Handbook

`bijux` is the root Bijux command runtime. It is the user-facing surface for
runtime health, official app routing, plugins, layered config, history,
memory, REPL behavior, and structured output.

Use this handbook when the question is about what `bijux` does at the command
line or how the Python distribution reaches the same runtime contract.

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="packages/bijux-cli.md">Open the runtime package</a>
<a class="md-button" href="packages/bijux-cli-python.md">Open the Python bridge package</a>
</div>

## What `bijux` Covers Today

The root `bijux --help` surface currently exposes these top-level command
groups:

- runtime and diagnostics: `status`, `audit`, `docs`, `doctor`, `version`,
  `install`, `explain`
- app routing: `apps`
- configuration and extension points: `config`, `plugins`
- interaction and local state: `repl`, `completion`, `history`, `memory`

Official apps such as `atlas`, `dag`, `dna`, `gnss`, `rag`, `rar`, and `vex`
mount through the runtime rather than redefining the runtime contract.

## Packages In This Surface

- [`bijux-cli`](packages/bijux-cli.md) handles native runtime semantics
- [`bijux-cli-python`](packages/bijux-cli-python.md) handles Python packaging
  and bridge compatibility
- stay in this handbook when the question spans both CLI packages

## Code Anchors

- `crates/bijux-cli/src/bin/bijux.rs`
- `crates/bijux-cli/src/bootstrap/run.rs`
- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/src/contracts/`

## Use This Handbook When

- the question is about `bijux` command behavior, flags, output, or exit codes
- plugin or mounted-app routing boundaries are in scope
- CLI and REPL behavior must stay aligned
- Python packaging must stay faithful to the native runtime
- a documentation claim needs to be verified against source and tests

## Main Paths

- [Foundation](foundation/index.md)
- [Architecture](architecture/index.md)
- [Interfaces](interfaces/index.md)
- [Operations](operations/index.md)
- [Quality](quality/index.md)

## Related Handbooks

- [Repository Handbook](../bijux-core/index.md)
- [DAG Handbook](../bijux-dag/index.md)
- [Maintainer Handbook](../bijux-dev/index.md)
