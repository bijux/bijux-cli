---
title: Data Contracts
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Data Contracts

This page explains the data shapes that let the CLI speak consistently to users,
scripts, plugins, and tests.

The main question is not where a struct lives. It is what kind of promise that
struct makes once it leaves one function boundary and becomes observable.

## Contract Map

```mermaid
flowchart LR
    contracts["cli contracts"] --> command["command and namespace"]
    contracts --> execution["execution policy"]
    contracts --> envelope["success and error envelopes"]
    contracts --> config["config state and mutation"]
    contracts --> plugin["plugin manifests"]
    contracts --> diagnostics["diagnostics records"]
```

## Contract Families

- command path and namespace normalization
- execution flags and policy representation
- success/error envelope payloads
- config read/write and mutation payloads
- plugin lifecycle and compatibility declarations
- diagnostics and route inventory records

## Code Anchors

- `crates/bijux-cli/src/contracts/mod.rs`
- `crates/bijux-cli/src/contracts/command.rs`
- `crates/bijux-cli/src/contracts/execution.rs`
- `crates/bijux-cli/src/contracts/envelope.rs`
- `crates/bijux-cli/src/contracts/plugin.rs`
- `crates/bijux-cli/src/contracts/diagnostics.rs`

## Contract Rules

- add validation at construction boundaries where possible
- keep schema-bearing structs serializable and reviewable
- treat field removals/renames as compatibility-impacting changes
- keep examples and docs aligned with actual struct behavior

## Reading Rule

Use this page when a CLI change starts crossing process, file, plugin, or test
boundaries and the question becomes which payload promise is being changed.

## Next Reads

- [Artifact Contracts](artifact-contracts.md)
- [Compatibility Commitments](compatibility-commitments.md)
- [Invariants](../quality/invariants.md)
