---
title: Deployment Boundaries
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Deployment Boundaries

Use this page when the CLI works in one host context but not another and you
need to know whether the problem belongs to `bijux`, the package channel, or an
adjacent delegated tool.

Deployment boundaries matter because `bijux-cli` lives inside a shared
repository and can delegate to other products or tools. Readers need a clear
line between what the CLI itself owns and what it merely discovers or hands
off to.

## Boundary Areas

- host environment assumptions for command and completion behavior
- package/channel alignment for runtime version identity
- delegated known-tool route handoff boundaries
- state directory and plugin path ownership boundaries

## Code Anchors

- `crates/bijux-cli/src/interface/cli/dispatch/delegation.rs`
- `crates/bijux-cli/src/features/install/compatibility.rs`
- `crates/bijux-cli/src/features/install/paths.rs`
- `crates/bijux-cli/src/features/diagnostics/state_paths.rs`

## What Responsibility Shifts Mean

| Surface | What `bijux-cli` owns |
| --- | --- |
| host path and state resolution | it should explain where it is reading and writing |
| package and channel identity | it should make runtime version identity inspectable |
| delegated known-tool routes | it should fail transparently when control leaves CLI ownership |
| plugin and state path boundaries | it should expose the directories and policies it expects |

## Boundary Rules

- document host limitations clearly and keep them current
- treat delegated-route failures as observable operational errors
- avoid implicit cross-package assumptions in CLI-only release notes

## Reader Shortcut

If a route crosses into another product or external tool, success still depends
on that adjacent surface being healthy. The CLI should make the handoff visible,
not pretend ownership it does not have.

## Continue Reading

- [Repository Fit](../foundation/repository-fit.md)
- [Integration Seams](../architecture/integration-seams.md)
- [Risk Register](../quality/risk-register.md)
