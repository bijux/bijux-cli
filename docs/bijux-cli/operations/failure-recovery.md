---
title: Failure Recovery
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Failure Recovery

Use this page when a CLI command fails and you need the safest route back to a
known-good state without erasing evidence or causing additional damage.

Failure recovery in `bijux-cli` starts with deterministic diagnostics, explicit
state inspection, and narrowly targeted remediation. The goal is to restore
trust, not just to make the error disappear.

## Recovery Workflow

1. Capture stderr payload and exit code.
2. Run `status`, `doctor`, and `audit` in structured mode.
3. Inspect state-path and plugin location reports.
4. Apply minimal remediation (`config`, `plugins`, `history`, `memory`).
5. Re-run the failing command and diagnostics checks.

## Recovery Commands

```bash
bijux status --format json --no-pretty
bijux doctor --format json --no-pretty
bijux audit --format json --no-pretty
bijux plugins doctor
bijux plugins explain
```

## What Recovery Should Protect

| Step | Why it matters |
| --- | --- |
| capture the failure | preserves the evidence you need for repeatable diagnosis |
| inspect state and plugin paths | prevents blind cleanup that hides the real cause |
| remediate one thing at a time | keeps causality understandable |
| rerun diagnostics and the failing command | proves the system is actually recovered |

## Code Anchors

- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs`
- `crates/bijux-cli/src/features/diagnostics/state_diagnostics.rs`
- `crates/bijux-cli/src/features/plugins/diagnostics.rs`

## Recovery Rules

- avoid broad state deletion without a bounded diagnosis
- preserve failing payloads for reproducible debugging
- apply one remediation step at a time for clear causality

## Reader Shortcut

If the recovery step is "delete things until it works," recovery quality is
already poor. A serious CLI should let you narrow the failure first, then fix
the smallest possible surface.

## Continue Reading

- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Risk Register](../quality/risk-register.md)
- [Change Validation](../quality/change-validation.md)
