# ACCEPTANCE RELEASE AND READINESS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/FOUNDATION_READINESS_CRITERIA.md
# Foundation readiness criteria

## Scope

Defines objective readiness criteria before new feature development resumes.

## Required criteria

- runtime architecture contracts are current and enforced by governance suites
- crate boundary policy has zero unresolved violations
- artifact hardening contract and integrity suite are present
- test trust catalog covers semantic, adversarial, failure, replay, scheduler, and recovery surfaces
- control-plane foundation suites are green in policy mode

## Mandatory evidence artifacts

- repository architecture report
- runtime module ownership report
- artifact contract report
- test trust coverage report
- foundation final report

## Exit rule

Feature development remains frozen until all criteria are marked satisfied in the foundation final report.

## SOURCE: docs/spec/MISSION_STATEMENT.md
# Mission Statement

## Canonical one-liner

`Git for computation graphs.`

## Canonical long-form mission

Bijux DAG is a deterministic computation-graph system that treats graph identity, run identity,
artifact lineage, replay fidelity, and diff explainability as first-class truth surfaces. The
repository prioritizes proof of behavior over breadth of platform claims.

## Drift policy

- Root docs must use the canonical one-liner and long-form wording intent.
- Alternative taglines are not allowed in root messaging.
- Release review must include mission/README drift validation.

## Related docs

- `README.md`
- `docs/reference/POSITIONING_NOTE.md`
- `docs/reference/GIT_FOR_COMPUTATION_GRAPHS_MAPPING.md`
- `docs/spec/CURRENT_IMPLEMENTED_CAPABILITIES.md`

## SOURCE: docs/spec/RELEASE_BINARY_VERIFICATION.md
# Release Binary Verification

## Verification suite
- verify version output:
  - `dag version --json`
- verify capabilities output:
  - `dag capabilities --json`
- verify dry command parsing:
  - `dag --help`
- verify inspection command availability:
  - `dag runs --help`

## Integrity policy
- Release artifacts must be checksumed.
- Signature policy is pending and tracked as release governance work.

## SOURCE: docs/spec/RELEASE_REVIEW_CHECKLIST.md
# Release Review Checklist

## Scope
Checklist for release approvers.

## Checklist
1. Public API surface review complete.
2. Run directory compatibility review complete.
3. Import/export format compatibility review complete.
4. Compatibility matrix generated and reviewed.
5. Benchmark regression report reviewed.
6. Resource profile regression report reviewed.
7. Known limitations section updated.
8. Reproducibility check report attached.
9. Post-release verification suite passed.
10. Mission and README drift review complete against `docs/spec/MISSION_STATEMENT.md`.
11. Crate-boundary regression review complete (responsibility gates and forbidden-edge report checked).
12. Evidence pruning review complete (stale note-only and duplicate evidence reports removed).
13. Performance claims are backed by raw benchmark data and scorecard references.

## Related tests
- `bijux-dev-dag release post-release-verify`

## Versioning and change policy
Checklist changes must remain aligned with `docs/spec/RELEASE_POLICY.md`.
