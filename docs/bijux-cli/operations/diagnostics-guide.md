---
title: Diagnostics Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Diagnostics Guide

Use this page when `bijux` behaves unexpectedly, an automation health check
needs structured evidence, or a support report must explain what the runtime
actually observed.

Diagnostics are a product surface, not a collection of debug conveniences.
`status`, `doctor`, `audit`, plugin diagnostics, and bounded telemetry answer
different questions. Use the narrowest surface that can either confirm the
suspected fault or produce evidence for the next investigation.

## Choose The Evidence Surface

| Command or surface | Best used for |
| --- | --- |
| `bijux status` | routine, machine-readable runtime and install health |
| `bijux doctor` | broad configuration, state, plugin, and install diagnosis |
| `bijux doctor paths` | wrong state, config, or plugin path resolution |
| `bijux doctor routing` | route inventory and dispatch confusion |
| `bijux doctor shims` | deprecated wrappers and `PATH` ambiguity |
| `bijux doctor python` | bridge availability and interpreter selection |
| `bijux doctor <app>` | health checks owned by one mounted application |
| `bijux plugins doctor` | plugin registry and lifecycle failures |
| `bijux plugins explain` | why a plugin was selected or rejected |
| `bijux audit` | consolidated check inventory and known issues |
| telemetry events | route flow and command completion timing |

Start with `status` for routine health checks. Move to `doctor` when the fault
involves configuration or environment state, and to the plugin-specific
commands when the evidence already points to plugin ownership. `audit` is an
inventory, not a substitute for a focused diagnosis.

## Capture A Reproducible Bundle

`bijux doctor --bundle` writes evidence under
`./artifacts/bijux-cli/doctor-bundle` so a report can preserve the observed
state without relying on terminal history. The bundle contains:

- `doctor.json`
- `docs.json`
- `config/generated-reference.md`

Run the command again when configuration or installed components change. A
bundle is a snapshot, not a live view, and should be attached to a report with
the command that failed and the smallest reproducible input.

## Read Telemetry Conservatively

Telemetry can record invocation start and finish, route completion,
unknown-route suggestions, and bounded command or message fields. Its sink is
opt-in and intended for local diagnosis. It does not replace command results or
the evidence bundle.

Treat telemetry as potentially sensitive operational data:

- enable it only for a defined investigation
- keep recorded fields bounded rather than copying arbitrary payloads
- inspect the sink before sharing it outside the machine
- disable it after the investigation when continuous collection is unnecessary

## Escalate Without Guessing

1. Re-run the failing command with the smallest input that still fails.
2. Capture `bijux status` and the relevant focused diagnostic command.
3. Use `bijux doctor --bundle` when the fault depends on machine state.
4. Record the expected result, observed result, CLI version, and exact command.
5. Preserve failing payloads and apply one remediation at a time.

Do not begin by deleting state broadly. If a claim cannot be checked through a
command result, `status`, `doctor`, `audit`, plugin diagnostics, or bounded
telemetry, identify that observability gap in the report rather than inventing
an explanation.

## Implementation Ownership

- `crates/bijux-cli/src/features/diagnostics/` owns diagnostic behavior.
- `crates/bijux-cli/src/interface/cli/handlers/cli.rs` owns CLI presentation.
- `crates/bijux-cli/src/interface/cli/dispatch.rs` owns route dispatch.
- `crates/bijux-cli/src/shared/telemetry.rs` owns telemetry boundaries.

Changes to these surfaces must preserve structured output used by automation
and keep optional telemetry bounded. A regression that prevents operators from
producing reliable evidence is an operational defect, even if the underlying
command still completes.

## Related Operations

- [Failure Recovery](failure-recovery.md)
- [Security and Safety](security-and-safety.md)
- [Risk Register](../quality/risk-register.md)
