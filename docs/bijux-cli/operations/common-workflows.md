---
title: Common Workflows
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Common Workflows

Use this page when you do not need the full command catalog, only the normal
way to get routine work done without creating avoidable confusion or state
drift.

The goal is not to enumerate every route. It is to show the sequences people
actually reach for when moving from inspection to intentional state change.

## Workflow Set

- runtime health: `status`, `doctor`, `audit`
- configuration: `config list/get/set/unset/export/load`
- memory state: `memory list/get/set/delete/clear`
- history management: `history --limit/--filter/--sort` and `history clear --force`
- plugin lifecycle: install, inspect, check, enable/disable, uninstall

## Example Session

```bash
bijux status --format json --no-pretty
bijux config set profile=dev
bijux memory set context=oncall
bijux history --limit 25 --sort timestamp
bijux plugins list
```

## What These Workflows Are For

| Workflow | What it should help you achieve |
| --- | --- |
| health | decide whether the CLI itself is healthy before changing anything |
| configuration | inspect and adjust runtime policy deliberately |
| memory and history | understand or reset local state with visible intent |
| plugins | manage extension lifecycle without losing track of trust and health |
| verification | confirm the runtime still looks coherent after a state change |

## Code Anchors

- `crates/bijux-cli/src/interface/cli/handlers/root.rs`
- `crates/bijux-cli/src/interface/cli/handlers/config.rs`
- `crates/bijux-cli/src/interface/cli/handlers/memory.rs`
- `crates/bijux-cli/src/interface/cli/handlers/history.rs`
- `crates/bijux-cli/src/interface/cli/handlers/plugins.rs`

## Workflow Rules

- prefer explicit options over implicit defaults in automation
- validate plugin health after each lifecycle mutation
- keep state changes observable with status and diagnostics checks

## Reader Shortcut

If a workflow changes CLI state and you cannot point to the verification step
afterward, the workflow is incomplete. Read-modify-verify is the normal safe
shape, not optional ceremony.

## Continue Reading

- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Review Checklist](../quality/review-checklist.md)
