---
title: Operator Workflows
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Operator Workflows

This page owns the safe operator lifecycle for local runtime state:
read, modify deliberately, then verify through an independent observation.

## Workflow Map

```mermaid
flowchart LR
    start["start session"] --> status["health checks"]
    status --> state["state review or config change"]
    state --> plugins["plugin management if needed"]
    plugins --> verify["audit and diagnostics follow-up"]
```

## Read, Modify, Verify

| Surface | Read | Modify | Verify |
| --- | --- | --- | --- |
| configuration | `config get`, `config explain`, `config validate` | `config set`, `config unset`, `config load` | rerun `config validate` and inspect the effective source |
| memory | `memory list`, `memory get` | `memory set`, `memory delete`, `memory clear` | read the affected key or list |
| history | `history` with explicit filters | `history clear --force` | query the same bounded history view |
| plugins | `plugins list`, `plugins inspect`, `plugins check` | install, enable, disable, uninstall | `plugins check` and focused runtime diagnostics |

Run `bijux doctor` before changing state when the failure domain is unclear.
Use structured output in automation, and treat a mutation without a separate
verification command as incomplete.

## Code Anchors

- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`
- `crates/bijux-cli/src/interface/cli/handlers/config.rs`
- `crates/bijux-cli/src/interface/cli/handlers/history.rs`
- `crates/bijux-cli/src/interface/cli/handlers/memory.rs`
- `crates/bijux-cli/src/interface/cli/handlers/plugins.rs`

## Next Reads

- [Common Workflows](../operations/common-workflows.md)
- [Diagnostics Guide](../operations/diagnostics-guide.md)
- [Review Checklist](../quality/review-checklist.md)
