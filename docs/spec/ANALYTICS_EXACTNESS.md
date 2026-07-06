---
title: Analytics Exactness
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Analytics Exactness

Multi-run analytics should be exact about what is visibly retained and honest
about what missing or corrupt evidence prevents them from proving.

## Scope

This document defines the exactness boundary for:

- `dag runs summary`
- `dag runs compare`
- `dag runs trend`
- `dag runs failures`
- `dag runs flakes`

## Exactness rules

- run count is exact with respect to directories visible under the selected run
  root
- status, retry, cache-hit, artifact, and failure-kind aggregates are exact only
  for parseable retained evidence
- graph-flake detection is exact only for retained runs whose graph fingerprint
  and status can be read from governed evidence
- compare reports exact field equality or difference for the two named runs, but
  corrupt fields remain `null` rather than guessed

## Conservative degradation

Malformed JSON in manifests or traces must not be repaired implicitly during an
analytics query. Instead, the analytics surface degrades conservatively:

- corrupt manifests become `null`-backed summary fields
- corrupt traces do not invent retries, cache hits, or failure kinds
- missing evidence yields `unknown`, `null`, or zero-derived fields where the
  implementation cannot prove a stronger statement

Filesystem access failures remain hard errors. Corrupt retained content remains
queryable as conservative output rather than an opportunistic rewrite.

## Related tests

- `crates/bijux-dag-app/tests/multi_run_analytics_contract.rs`
- `crates/bijux-dag-app/src/inspect/run_views.rs`

## Versioning and change policy

Any incompatible change to exactness semantics, conservative degradation rules,
or hard-error boundaries must update this document and the linked tests in the
same change.
