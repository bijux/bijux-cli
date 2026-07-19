---
title: Test Trust Contract
audience: maintainer
type: specification
status: canonical
owner: bijux-core-quality
last_reviewed: 2026-07-19
---

# Test Trust Contract

## Scope

This contract governs how the repository connects supported behavior to named
tests and prevents test inventory from being mistaken for semantic proof.

## Authorities

| Authority | Responsibility |
| --- | --- |
| `TEST_PHILOSOPHY.md` | human test-design and evidence standard |
| `configs/dag/policy/test_trust_ledger.json` | policy classification, critical tests, required semantic surfaces, and coverage families |
| `crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json` | runtime test inventory grouped by trust class |
| `crates/bijux-dev/tests/test_trust_maintenance_contracts.rs` | executable policy and reference validation |
| `docs/reports/foundation/TEST_TRUST_COVERAGE_REPORT.md` | reviewed coverage assessment derived from the ledger and executable maintenance evidence |
| `docs/reports/foundation/TEST_TRUST_MAINTENANCE_REPORT.md` | generated maintenance observation |

The policy ledger is normative. Reports are revision-specific evidence and
cannot redefine classification rules. The coverage report is curated review
evidence because no repository command generates it; it must not claim
generated provenance. The maintenance report is generated and remains owned by
its producer.

## Required Properties

- Every cataloged test path exists.
- Every required semantic surface maps to an existing runtime test.
- Every trust family is nonempty and maps to existing tests.
- Critical tests are explicitly listed in `must_never_break`.
- Snapshot macros appear only in existing allowlisted files.
- Cosmetic or duplicate classifications cannot support release-blocking claims.
- Advisory, filtered, ignored, simulated, and platform-limited results remain
  visible in evidence.

Classification does not make a weak test strong. Reviewers still verify that
the test reaches the claimed behavior and asserts a meaningful result.

## Trust Classes

| Class | Meaning |
| --- | --- |
| critical | protects behavior whose regression invalidates a supported runtime or safety claim |
| useful | provides meaningful behavior or boundary evidence but is not the sole proof of a release-critical claim |
| shallow | reaches a surface but needs stronger semantic assertions or broader failure coverage |
| cosmetic | verifies presentation without product semantics |
| duplicate | repeats another test without an independent trust property |

Shallow, cosmetic, and duplicate tests are visible debt. They are not removed
merely to improve a count; they are strengthened, consolidated, or deleted
with review of the behavior they currently cover.

## Change Contract

A new supported runtime behavior requires:

1. an owning specification or package contract;
2. a named executable proof;
3. ledger and catalog classification when it belongs to a governed family;
4. failure or adversarial evidence appropriate to its risk;
5. generated trust reports refreshed from their producer and curated
   assessments reviewed against the resulting ledger and test evidence.

Removing or renaming a test requires updating every ledger, catalog, report
producer, and handbook reference in the same change.

## Failure Meaning

A missing test path is a governance failure. A passing catalog validator proves
reference integrity, not behavior correctness. A stale generated report or
curated assessment is an evidence failure. A release claim with no critical
proof is unsupported even when unrelated suites are green.

## Versioning

Trust classes, required semantic surfaces, and the meaning of
`must_never_break` are stable governance interfaces. Incompatible changes
require policy, tests, reports, and maintainer documentation to change
together.
