---
title: Observability and Diagnostics
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Observability and Diagnostics

Use this page when you need to understand how `bijux` makes its own behavior
inspectable, whether for local debugging, automation health checks, or support
triage.

Observability in `bijux-cli` is a combination of structured command payloads,
purpose-built diagnostics commands, and optional telemetry. The point is to
make runtime behavior legible without guessing at hidden state.

## Diagnostic Surfaces

- `status`: runtime, state, plugins, and install summary
- `doctor`: configuration, install, state, plugin, Python bridge, and app-mount health checks
- `doctor --bundle`: write a reproducible diagnostics bundle under `./artifacts`
- `audit`: consolidated check inventory and issues
- `plugins doctor` and `plugins explain`: plugin-specific diagnostics

## Telemetry Surface

- invocation start/finish events
- route completion and unknown-route suggestion events
- bounded command and message field recording
- opt-in sink configuration for local diagnostics

## What Each Surface Is Good At

| Surface | Best used for |
| --- | --- |
| `status` | routine machine-readable health checks |
| `doctor` | configuration and environment diagnosis |
| `doctor --bundle` | packaging evidence for support and later inspection |
| `audit` | summarizing known issues in one place |
| telemetry events | understanding route flow and command completion timing |

## Code Anchors

- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/shared/telemetry.rs`
- `crates/bijux-cli/src/features/diagnostics/`
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`

## Diagnostics Rules

- prefer machine-readable output for automation and CI checks
- keep telemetry optional and bounded to avoid leaking oversized data
- treat diagnostics regressions as operational blockers

## Reader Shortcut

If an operational claim cannot be checked through `status`, `doctor`, `audit`,
or a bounded telemetry surface, the CLI is harder to trust than it should be.
Observability is part of the product, not an afterthought.

## Continue Reading

- [Failure Recovery](failure-recovery.md)
- [Risk Register](../quality/risk-register.md)
- [Security and Safety](security-and-safety.md)
