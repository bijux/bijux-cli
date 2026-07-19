---
title: Test Trust Ledger
audience: maintainer
type: specification
status: canonical
owner: bijux-core-quality
last_reviewed: 2026-07-19
---

# Test Trust Ledger

## Scope

`configs/dag/policy/test_trust_ledger.json` is the machine-readable authority
for runtime test classification, critical coverage, required semantic
surfaces, snapshot policy, and trust-family maintenance.

## Ownership

The runtime foundation owns behavioral classification. The `bijux-dev`
control plane owns reference validation and report generation. Neither owner
may change a classification merely to make a gate pass.

## Required Sections

| Section | Contract |
| --- | --- |
| `classification_rules` | ordered matching from specific critical files to broader useful patterns and named debt |
| `must_never_break` | exact critical test files required for supported runtime claims |
| `required_semantic_surfaces` | one existing proof for each named high-risk behavior |
| `snapshot_surface_policy` | existing files allowed to use governed snapshot macros |
| `trust_coverage_families` | nonempty behavior families mapped to existing tests |

Specific file classifications take precedence over broad filename patterns.
An unmatched test is a maintenance signal; it is not implicitly critical or
implicitly safe to ignore.

## Snapshot Policy

Snapshot allowlist entries must exist. Exact snapshots are permitted only when
serialized or rendered shape is itself the contract. Tests should prefer typed
semantic assertions where field-level meaning matters.

Removing the final governed snapshot macro from a file requires removing its
allowlist entry. An allowlist is permission for a reviewed use, not a target
list that requires snapshots.

## Maintenance

When a runtime test is added, renamed, merged, or removed:

1. classify its trust property;
2. update critical and semantic-surface mappings when applicable;
3. update each affected family;
4. run the maintenance contract;
5. regenerate trust coverage and maintenance reports;
6. inspect whether the resulting claim became stronger, weaker, or merely
   reorganized.

Counts alone do not determine that conclusion.

## Failure Meaning

- Missing paths mean the ledger is stale.
- Empty families mean the policy claims coverage without a proof.
- A critical behavior absent from `must_never_break` is ungoverned.
- A listed test that does not reach its behavior is false confidence even when
  reference checks pass.
- A stale report is not corrected by changing the ledger to match it.

## Verification

Run:

```bash
cargo test --locked -p bijux-dev --test test_trust_maintenance_contracts
```

The generated trust reports are supporting evidence and must be refreshed by
their owning maintainer command when policy or classified tests change.
