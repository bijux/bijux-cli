# ADR: Repo-Tree Shape Target Before v0.1.0

- Date: 2026-03-08
- Status: Accepted

## Context

Repository growth produced very large modules, tiny wrappers, and uneven ownership visibility. We need explicit hygiene boundaries that are enforceable and reviewable.

## Decision

Adopt module hygiene governance policy at `configs/policy/module_hygiene_governance.json`.

Required rules:
- new top-level modules require ownership classification
- new large modules require split rationale
- new tiny wrappers require justification
- module hygiene drift is enforced via `make module-hygiene-drift`

Required outputs:
- under-10-line module review
- over-500-line module inventory
- over-1000-line module inventory
- zero-direct-test module inventory
- repo-tree hotspots and cleanup candidate reports
- maintainer repo-tree health dashboard

## Consequences

- module growth is governed by explicit thresholds
- ownership and decomposition debt is visible in generated reports
- release workflow gains a deterministic hygiene drift gate
