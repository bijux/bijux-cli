---
title: Installation and Setup
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Installation and Setup

Use this page when you need to install `bijux` and prove that the binary,
resolved paths, and diagnostics surfaces are trustworthy before any automation
or daily usage begins.

A good setup result is not just "the command exists on PATH." It means one
clear runtime binary is active, state locations are visible, and the CLI can
describe its own health without ambiguity.

## Setup Checklist

1. Install the runtime from the chosen channel.
2. Confirm active binary and version identity.
3. Verify resolved state paths and plugin registry location.
4. Run diagnostics commands before script usage.

## Baseline Commands

```bash
bijux version
bijux status --format json --no-pretty
bijux cli paths
bijux doctor
bijux audit
```

## What These Checks Should Tell You

| Check | What it should confirm |
| --- | --- |
| `bijux version` | the invoked binary is the one you expect to trust |
| `bijux status` | runtime identity, state, and plugin context look sane |
| `bijux cli paths` | config, state, and plugin directories resolve where you think they do |
| `bijux doctor` | install, config, bridge, and routing health are coherent |
| `bijux audit` | the CLI is not already reporting known operational problems |

## Code Anchors

- `crates/bijux-cli/src/features/install/diagnostics.rs`
- `crates/bijux-cli/src/features/install/query.rs`
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`
- `crates/bijux-cli/src/features/diagnostics/state_paths.rs`

## Setup Rules

- avoid multiple active binaries on `PATH`
- keep `status` and `doctor` clean before onboarding automation
- treat path-shadowing warnings as setup failures until resolved

## Reader Shortcut

If `bijux` works only until you ask it where its state lives or which binary is
active, the installation is not complete. Diagnose setup first, then automate.

## Continue Reading

- [Local Development](local-development.md)
- [Failure Recovery](failure-recovery.md)
- [Security and Safety](security-and-safety.md)
