---
title: History Retention Policy
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# History Retention Policy

Run history remains trustworthy only when retention removes or preserves whole
evidence units instead of partially rewriting them.

## Scope

This policy governs the retained run directories that feed:

- `dag runs history`
- `dag runs summary`
- `dag runs compare`
- `dag runs trend`
- `dag runs failures`
- `dag runs flakes`

## Retention boundary

The authoritative history unit is one finalized run directory. Retention may:

- preserve the run directory unchanged
- delete the run directory as a whole under an explicit retention action

Retention must not:

- rewrite `manifest.json`
- rewrite `outputs.index.json`
- rewrite `nodes/*/trace.json`
- mutate finalized evidence to make analytics look cleaner

## Derived index boundary

`.bijux-run-history-index.json` is a derived acceleration artifact. It can be
rebuilt from retained run directories and is not authoritative evidence.
Deleting or regenerating the index does not change run history; mutating
authoritative run evidence does.

## Partial history handling

Analytics and history queries must tolerate incomplete or corrupt retained
history conservatively. A damaged run directory is still part of the visible
history set until retention removes it, but derived fields may downgrade to
`null`, `unknown`, or zero-valued aggregates where governed evidence is absent.

## Related tests

- `crates/bijux-dag-app/tests/multi_run_analytics_contract.rs`
- `crates/bijux-dag-app/src/inspect/run_views.rs`
- `docs/spec/RUN_DIR_CONTRACT.md`
- `docs/spec/RUN_DIR_STORAGE_CONTRACT.md`

## Versioning and change policy

Any incompatible change to retained-history ownership, derived-index authority,
or allowed retention actions must update this policy and the linked contracts in
the same change.
