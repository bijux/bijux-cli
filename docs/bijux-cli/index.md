---
title: CLI Handbook
audience: mixed
type: index
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-07
---

# CLI Handbook

`bijux` is the root command runtime for the wider Bijux tool ecosystem. It is
the operator-facing surface for mounted apps, plugins, layered config,
diagnostics, history, memory, REPL behavior, and structured output.

Use this handbook when the question is about what `bijux` does at the command
line, how it behaves under automation, or how the Python distribution reaches
the same runtime contract.

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="packages/bijux-cli.md">Open the runtime package</a>
<a class="md-button" href="packages/bijux-cli-python.md">Open the Python bridge package</a>
</div>

## What `bijux` Covers Today

The visible `bijux --help` surface currently groups into four kinds of work:

- runtime and diagnostics: `status`, `audit`, `docs`, `doctor`, `version`,
  `install`, `explain`
- app routing: `apps`
- configuration and extension points: `config`, `plugins`
- interaction and local state: `repl`, `completion`, `history`, `memory`

Official apps such as `atlas`, `dag`, `dna`, `gnss`, `rag`, `rar`, and `vex`
mount through this runtime rather than defining their own root command
contract.

## Start Here

| If you want to... | Open this page |
| --- | --- |
| understand what `bijux` promises at the command line | [Interfaces](interfaces/index.md) |
| understand how the runtime is assembled | [Architecture](architecture/index.md) |
| install, diagnose, or operate the CLI locally | [Operations](operations/index.md) |
| understand package roles and product scope | [Foundation](foundation/index.md) |
| review test-backed limits, invariants, and acceptance standards | [Quality](quality/index.md) |

## What This Handbook Keeps Straight

- what belongs to the root runtime instead of an app, plugin, or maintainer
  surface
- how CLI, REPL, plugin, and config behavior stay aligned
- where `bijux-cli` ends and `bijux-cli-python` begins
- which behavior is part of the user-facing contract and which belongs to
  internal assembly

## Package Split

- [`bijux-cli`](packages/bijux-cli.md) owns native runtime semantics,
  routing, execution flow, and output behavior
- [`bijux-cli-python`](packages/bijux-cli-python.md) owns Python packaging,
  launcher behavior, and bridge compatibility

Stay in this handbook when the question spans both packages or when the right
owner is not obvious yet.

## Good First Reads

- Read [CLI Surface](interfaces/cli-surface.md) for the command-level contract.
- Read [Configuration Guide](interfaces/config-guide.md) for layered config
  behavior and operator workflows.
- Read [Diagnostics Guide](operations/diagnostics-guide.md) when the question
  starts with `bijux doctor` or runtime health.
- Read [Python Bridge Package](packages/bijux-cli-python.md) when install or
  launcher behavior is part of the problem.

## When To Leave This Handbook

- Move to the [Repository Handbook](../bijux-core/index.md) when the answer
  depends on shared release rules or cross-product ownership.
- Move to the [DAG Handbook](../bijux-dag/index.md) when the question is about
  graph execution or the `bijux-dag` surface rather than the root runtime.
- Move to the [Maintainer Handbook](../bijux-dev/index.md) when the question
  is about repository gates, documentation generation, or release proof.
