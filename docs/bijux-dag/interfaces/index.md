---
title: Interfaces
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# DAG Interfaces

Use this section when you need the supported caller contract for `bijux-dag`:
commands, public crate imports, configuration and policy inputs, retained
artifact payloads, and compatibility boundaries.

Start here when you are authoring workflows, integrating DAG execution into a
tool, automating around retained run data, or checking whether a route belongs
to the shipped public `v0.4.0` surface.

## Start With The Question You Have

| If you need to... | Open this page |
| --- | --- |
| inspect the visible `bijux-dag` command contract | [CLI Surface](cli-surface.md) |
| see real runnable examples before adopting the tool | [Runnable Examples](examples/index.md) |
| embed DAG behavior from Rust | [API Surface](api-surface.md) |
| understand graph, run, replay, and diff payloads | [Data Contracts](data-contracts.md) |
| understand compatibility promises and boundaries | [Compatibility Commitments](compatibility-commitments.md) |
| inspect hidden or intentionally gated routes | [Gated Command Inventory](reference/gated-command-inventory.md) |

## What This Section Covers

- the public `bijux-dag --help` command surface
- stable Rust-facing imports across the public DAG crates
- runtime, policy, and configuration inputs that affect execution
- graph, run, artifact, replay, and comparison payload contracts
- deliberately hidden routes only when the question is about internal,
  simulated, or experimental coverage

## Code Anchors

- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/commands/mod.rs`
- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`
- `crates/bijux-dag-artifacts/src/lib.rs`

## Pages In This Section

- [CLI Surface](cli-surface.md)
- [Generated CLI Reference](generated-cli-reference.md)
- [Command Taxonomy](command-taxonomy.md)
- [Operator Command Index](operator-command-index.md)
- [Operator Inspection Guide](operator-inspection-guide.md)
- [Support Matrix](support-matrix.md)
- [Runnable Examples](examples/index.md)
- [API Surface](api-surface.md)
- [Configuration Surface](configuration-surface.md)
- [Data Contracts](data-contracts.md)
- [Compatibility Matrix](compatibility-matrix.md)
- [Graph Schema Reference](reference/graph-schema.md)
- [Reproducibility Model](reference/reproducibility-model.md)
- [Run Evidence Layout](reference/run-evidence-layout.md)
- [Error Codes](error-codes.md)
- [Gated Command Inventory](reference/gated-command-inventory.md)
- [Authoring Guide](authoring-guide.md)
- [Reusable Subgraphs](guides/reusable-subgraphs.md)
- [Artifact Contracts](artifact-contracts.md)
- [Entrypoints and Examples](entrypoints-and-examples.md)
- [Executable Recipes](executable-recipes.md)
- [Operator Workflows](operator-workflows.md)
- [Public Imports](public-imports.md)
- [Compatibility Commitments](compatibility-commitments.md)

## Public Boundary In Plain Terms

The default product story is local-first DAG execution. Validation, planning,
running, replay, comparison, cache inspection, and retained evidence all
belong to that story. Experimental, simulated, and maintainer-only lanes exist
in the repository, but they are not part of the normal operator contract unless
you enter them deliberately.

## Before You Move Deeper

- Stay in this section when the question is what operators, tools, or other
  crates may rely on.
- Move to Architecture when the next question is engine structure, scheduler
  assembly, or internal crate wiring.
- Move to Operations when the next question is how to run, diagnose, recover,
  or release real DAG workflows.
- Use `bijux-dag commands --lane experimental`,
  `bijux-dag commands --lane simulated`, or
  `bijux-dag commands --lane internal` when you need deliberate visibility into
  non-default routes instead of treating them as part of the public API.
