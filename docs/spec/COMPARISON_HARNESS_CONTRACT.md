---
title: Comparison Harness Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Comparison Harness Contract

The comparison harness exists to record evidence-backed scenario comparisons for
`bijux-dag` without turning interpretation or marketing language into product
truth.

## Scope

This contract governs comparison scenario metadata, baseline reports, factual
versus descriptive classification, release-blocking comparison gates, and the
documentation surfaces that explain how to read the results.

## Required evidence set

- `evidence/compare/metadata.json` is the canonical scenario registry
- `evidence/compare/scenarios/` contains the executable scenario records
- `evidence/compare/baselines/bijux_v1.json` is the baseline snapshot for the
  current governed harness
- every scenario must point to a real `bijux` evidence asset

## Scenario rules

- factual scenarios must measure the `bijux` side directly
- descriptive scenarios may explain interpretation limits, but they do not act
  as release evidence
- every scenario must declare non-equivalence limits
- release-blocking scenarios must also set `measured_bijux_side` to `true`
- the harness must keep at least five factual scenarios and at least five
  canonical scenario files

## Reading rule

Comparison output may describe equivalence classes, scenario limits, and
operator-facing interpretation. It must not make broad ranking claims without a
named evidence surface under `evidence/compare/`.

## Related surfaces

- `docs/bijux-dag/interfaces/comparison-report-format.md`
- `docs/bijux-dag/quality/comparison-limitations.md`
- `docs/bijux-dag/quality/comparison-evidence-surfaces.md`

## Related tests

- `crates/bijux-dag-app/tests/comparison_harness_contract.rs`

## Versioning and change policy

Any incompatible change to scenario classes, release-blocking semantics,
baseline format, or evidence interpretation rules must update this contract and
the linked proof surfaces in the same change.
