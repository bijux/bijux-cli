---
title: Common Workflows
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Common Workflows

Use this page to route an operational question to its owning guide. The
[operator lifecycle](../interfaces/operator-workflows.md) defines the common
read-modify-verify discipline; these pages own the detailed behavior.

## Workflow Index

| Need | Start here | Evidence to retain |
| --- | --- | --- |
| establish runtime health | [Diagnostics Guide](diagnostics-guide.md) | focused findings, resolved paths, and bundle location when exported |
| install and initialize the runtime | [Installation And Setup](installation-and-setup.md) | version, executable resolution, and initial doctor result |
| recover from corrupted or conflicting state | [Failure Recovery](failure-recovery.md) | original diagnosis, backup path, repair result, and final validation |
| understand configuration precedence | [Configuration Surface](../interfaces/configuration-surface.md) | effective value and source chain |
| operate plugins safely | [Operator Workflows](../interfaces/operator-workflows.md) | pre-change inventory and post-change health check |
| automate command consumption | [CLI Surface](../interfaces/cli-surface.md) | stable command path, structured envelope, and exit behavior |

## Code Anchors

- `crates/bijux-cli/src/interface/cli/handlers/root.rs`
- `crates/bijux-cli/src/interface/cli/handlers/config.rs`
- `crates/bijux-cli/src/interface/cli/handlers/memory.rs`
- `crates/bijux-cli/src/interface/cli/handlers/history.rs`
- `crates/bijux-cli/src/interface/cli/handlers/plugins.rs`

## Continue Reading

- [Diagnostics Guide](diagnostics-guide.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Review Checklist](../quality/review-checklist.md)
