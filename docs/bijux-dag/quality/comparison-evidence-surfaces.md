---
title: Comparison Evidence Surfaces
audience: mixed
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Comparison Evidence Surfaces

`bijux-dag` comparison claims are valid only when they point to a governed
scenario, a measured Bijux evidence asset, and explicit non-equivalence limits.
This page explains the evidence set and how far its conclusions may travel.
[Comparison Harness Contract](../../spec/COMPARISON_HARNESS_CONTRACT.md)
governs the machine and test surfaces.

## Evidence Authority

| Surface | Responsibility |
| --- | --- |
| `evidence/compare/metadata.json` | scenario registry, classification, measured evidence, limits, and release role |
| `evidence/compare/scenarios/` | canonical scenario records |
| `evidence/compare/baselines/bijux_v1.json` | governed baseline snapshot for the current harness |
| referenced `bijux_evidence_asset` | direct repository evidence for the Bijux side of a factual scenario |
| `crates/bijux-dag-app/tests/comparison_harness_contract.rs` | executable report and scenario contract |
| `crates/bijux-dev/src/commands/compare_evidence.rs` | metadata completeness and documentation policy gate |

The metadata registry is the entrypoint. A scenario file without a registry
entry, or a prose claim without a scenario, is not governed comparison
evidence.

## Scenario Classes

| Class | What it may support | What it may not support |
| --- | --- | --- |
| factual | a conclusion about directly measured Bijux behavior in the named scenario | claims outside the measured inputs, outputs, and declared limits |
| descriptive | context, scope, and interpretation guidance | release evidence or a measured equivalence claim |

Every release-blocking scenario must be factual and set
`measured_bijux_side` to `true`. The current registry has no license to turn
commentary, familiarity, or product positioning into release evidence.

## What The Harness Measures

Current factual scenarios cover bounded behavior such as:

- graph dependency shapes and failure propagation
- retry, timeout, cache reuse, and replay behavior
- deterministic retained evidence
- operator inspection and failure diagnostics
- scheduler overhead for the named tiny-task scenario
- artifact inspection and traceability

Each conclusion remains scoped to its scenario and evidence asset. Similarity
in one behavior does not establish whole-product equivalence.

## Non-Equivalence Limits

Every scenario must declare limits. Common exclusions include:

- vendor queue internals and external scheduler implementation
- cluster autoscaling and capacity-management economics
- hardware profile and container-runtime variance
- UI ergonomics, IDE integration, and language-specific stack traces
- storage backend layout and binary serialization internals

These are part of the result, not optional caveats. A report that drops its
limits overstates the evidence.

## Interpretation Rules

- Do not convert a scenario result into a blanket product ranking.
- Do not compare unmeasured internals by inference from public outcomes.
- Distinguish equivalence of observed behavior from equivalence of
  implementation.
- Keep environment and fixture identity with reported measurements.
- Treat a baseline as a governed regression reference, not timeless external
  truth.
- Use [Comparison Report Format](../interfaces/comparison-report-format.md) for
  the output schema consumed by automation.

## Release Use

Comparison evidence may affect release confidence only when the scenario is
factual, the Bijux side is measured, the referenced evidence exists, and the
declared limits remain visible in the report. Descriptive scenarios can explain
scope but cannot block or approve a release.

## Maintenance Contract

When a comparison scenario is added, removed, or reclassified, update the
metadata registry, scenario record, referenced evidence, baseline when
applicable, contract, and executable tests in the same change. The policy gate
requires at least five factual scenarios and rejects release-blocking entries
whose Bijux side is not measured.
