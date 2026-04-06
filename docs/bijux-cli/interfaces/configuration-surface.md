---
title: Configuration Surface
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Configuration Surface

Configuration behavior is exposed through `config` and `cli config` routes with
normalized keys, ASCII-safe values, and deterministic import/export behavior.

## Visual Summary

```mermaid
flowchart LR
    command["config command"] --> validate["key value validation"]
    validate --> storage["config storage operations"]
    storage --> result["structured command result"]
    storage --> paths["resolved state paths"]
```

## Configuration Commands

- `config` / `config list`
- `config get KEY`
- `config set KEY=VALUE`
- `config unset KEY`
- `config clear`
- `config reload`
- `config export PATH`
- `config load PATH`

## Contract Rules

- keys must be ASCII and normalized
- values must remain ASCII and control-character safe
- import/export uses dotenv-compatible key-value syntax
- command results should include status and path context where relevant

## Code Anchors

- `crates/bijux-cli/src/interface/cli/handlers/config.rs`
- `crates/bijux-cli/src/features/config/operations.rs`
- `crates/bijux-cli/src/features/config/validation.rs`
- `crates/bijux-cli/src/contracts/config.rs`

## Next Reads

- [State and Persistence](../architecture/state-and-persistence.md)
- [Common Workflows](../operations/common-workflows.md)
- [Change Validation](../quality/change-validation.md)
