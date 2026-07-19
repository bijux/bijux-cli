---
title: Operator Command Index
audience: operators
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Operator Command Index

Use this page after a run exists and the question is how to inspect, compare,
verify, diagnose, or retain its evidence. It groups the stable `runs` family by
operator intent and gives an escalation order that avoids opening files
manually before the CLI has classified them.

## Choose By Question

| Operator question | Command | Evidence returned |
| --- | --- | --- |
| Which runs exist? | `bijux-dag runs list` | retained run identities under an explicit root |
| What happened at a glance? | `bijux-dag runs show` | compact status and timing for one run |
| What evidence is available? | `bijux-dag runs inspect` | structured run, node, artifact, and compatibility summary |
| How was the graph shaped? | `bijux-dag runs tree` | retained node and dependency structure |
| In what order did execution occur? | `bijux-dag runs timeline` | ordered events from `observability.timeline.json`, with trace projection only as a compatibility fallback |
| Why did scheduling pause or choose a batch? | `bijux-dag runs scheduler-checkpoint` | ready, blocked, inflight, completed, and decision evidence |
| What first caused the failure? | `bijux-dag runs explain-failure` | first causal failure and downstream affected nodes |
| Is the retained run complete and trustworthy? | `bijux-dag runs verify` | integrity and compatibility verdict |
| Why is retained evidence unhealthy? | `bijux-dag runs doctor` | corruption, incompleteness, and unsupported-format diagnosis |
| What changed between two runs? | `bijux-dag runs diff` | retained run-directory differences |

## History And Comparison

Use the wider retained-history lane only after one-run inspection:

- `bijux-dag runs history` filters and selects retained history records.
- `bijux-dag runs summary` aggregates one repository-local overview.
- `bijux-dag runs compare` attributes status, retries, cache hits, timing,
  fingerprints, graph inputs, selected nodes, output hashes, and the first
  meaningful divergence.
- `bijux-dag runs trend` renders one analytics point per retained run.
- `bijux-dag runs failures` aggregates failed node kinds.
- `bijux-dag runs flakes` identifies graph fingerprints with mixed outcomes.
- `bijux-dag runs index` rebuilds or reads the retained history index.
- `bijux-dag runs diagnostics-bundle` packages bounded support evidence.

`compare` is attribution over governed evidence, not an unrestricted recursive
directory comparison. Missing or incompatible evidence must remain visible in
the result instead of being treated as equality.

## Evidence-First Sequence

1. Pass `--root` explicitly and run `bijux-dag runs list`.
2. Use `show` for a compact status check.
3. Use `inspect` before opening retained files directly.
4. Use `timeline` for event order or `scheduler-checkpoint` for scheduler state.
5. Use `explain-failure` when the run failed.
6. Use `verify` when integrity is in question.
7. Use `doctor` when verification reports unsupported, corrupt, or incomplete
   evidence.

These states are not interchangeable:

- `unsupported` means the reader refuses the retained format or version
- `corrupt` means governed evidence contradicts its integrity contract
- `incomplete` means required evidence is absent
- a failed workflow may still have structurally valid retained evidence

## Automation Rules

- Prefer `--json` for scripts and preserve the complete response envelope.
- Do not parse human tables or log wording as a stable contract.
- Treat unknown state and unknown error identifiers as failures.
- Keep the explicit run root with exported diagnostics so evidence can be
  traced to the correct storage boundary.
- Use [Run Evidence Layout](run-evidence-layout.md) when a command result must
  be reconciled with an exact retained path.

## Recovery Boundary

The public operator lane is inspect-first. `runs stop` is an explicit active-run
control, not a retained-evidence repair command. Mutating repair and maintainer
recovery surfaces remain gated rather than part of the default stable
inspection path.

## Contract Sources

- [Generated CLI Reference](generated-cli-reference.md) is generated from the
  stable Clap surface.
- [Gated Command Inventory](gated-command-inventory.md) lists experimental,
  simulated, and internal routes.
- [Operator Inspection Contract](../../spec/OPERATOR_INSPECTION_CONTRACT.md)
  governs evidence states and scenarios.
- `crates/bijux-dag-app/tests/operator_ux_contract.rs` executes imported,
  unsupported, corrupt, and timing-coherence cases.
