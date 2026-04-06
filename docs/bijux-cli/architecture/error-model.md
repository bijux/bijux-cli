---
title: Error Model
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Error Model

`bijux-cli` distinguishes usage failures, validation failures, plugin/runtime
failures, and internal execution failures. Error shaping is part of the command
contract because scripts depend on both payload shape and exit code.

## Visual Summary

```mermaid
flowchart TD
    failure["runtime failure"] --> classify["error classification"]
    classify --> usage["usage and parse errors"]
    classify --> validation["validation and config errors"]
    classify --> plugin["plugin lifecycle and load errors"]
    classify --> internal["internal execution errors"]
    usage --> exit2["exit code 2"]
    internal --> exit1["exit code 1"]
```

## Error Surfaces

- parser errors for invalid flags, subcommands, and argument forms
- route errors for reserved/conflicting/unknown/ambiguous namespaces
- handler-level validation errors for config, history, memory, and plugin commands
- rendering errors when requested output cannot be encoded safely

## Code Anchors

- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/interface/cli/dispatch/policy.rs`
- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/src/contracts/envelope.rs`

## Error Contract Rules

- include a clear user-facing message in stderr payloads
- preserve usage-class failures for user-input mistakes
- avoid reclassifying stable errors without compatibility review
- keep unknown-route suggestions deterministic and bounded

## Next Reads

- [Data Contracts](../interfaces/data-contracts.md)
- [Failure Recovery](../operations/failure-recovery.md)
- [Invariants](../quality/invariants.md)
