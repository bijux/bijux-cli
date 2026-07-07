---
title: Comparison Evidence Surfaces
audience: mixed
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Comparison Evidence Surfaces

The comparison harness draws from a governed set of metadata, scenarios,
baselines, and executable app contracts.

## Canonical surfaces

- metadata registry: `evidence/compare/metadata.json`
- canonical scenarios: `evidence/compare/scenarios/`
- governed baseline: `evidence/compare/baselines/bijux_v1.json`
- executable app proof: `crates/bijux-dag-app/tests/comparison_harness_contract.rs`
- maintainer policy gate: `crates/bijux-dev/src/commands/compare_evidence.rs`

## Evidence interpretation split

- factual scenarios measure `bijux` behavior directly
- descriptive scenarios may describe context or scope limits
- release-blocking comparison evidence must come from factual measured
  scenarios

## Maintenance rule

When a comparison scenario is added, removed, or reclassified, update the
metadata registry, the baseline if needed, and the harness contract docs in the
same change.
