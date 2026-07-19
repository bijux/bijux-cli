---
title: Definition Of Done
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Definition Of Done

A DAG change is done when the repository contains the intended behavior, the
evidence supports the stated claim, and another maintainer can understand the
remaining boundary without reconstructing the work.

## Completion Contract

| Area | Required completion evidence |
| --- | --- |
| ownership | implementation remains in the responsible crate and module; dependency direction is valid |
| behavior | focused tests prove success, refusal, and relevant failure semantics |
| compatibility | affected schemas, identities, retained evidence, outputs, imports, or commands have fixtures, migration, refusal, or release treatment |
| documentation | operator and maintainer guidance agree with shipped behavior and link to the owning authority |
| generated material | named producer was run, generated diff was reviewed, and source identity is retained |
| validation | exact commands and terminal status are recorded with omitted lanes named |
| repository state | logical commits are reviewable and no unrelated or uncommitted work remains |
| risk | known limitation, security boundary, performance regression, or deferred work is stated with owner and release posture |

Not every change needs every repository gate. Every change does need evidence
from each affected row.

## Release-Facing Additions

A change that affects a public crate, binary, stable command, retained format,
or release artifact is not done until:

- package and public API boundaries are reviewed;
- clean-tree package and publish dry-run evidence passes where applicable;
- command and schema references are regenerated;
- compatibility and known-limitations records are current;
- release notes state the shipped claim without promoting gated behavior.

`configs/dag/release/release_checklist.json` and
`configs/dag/release/release_blockers.json` are machine-readable release
inputs. They complement this completion contract; they do not replace code
review.

## Explicit Non-Completion

Work is not done when:

- only the new happy path is tested;
- an old fixture was deleted instead of migrated or refused;
- a failing contract was weakened to accept current output;
- a generated report exists but its producer, source, or final status is
  unknown;
- a broad test process started but has no terminal summary;
- docs describe intended behavior that the implementation does not expose;
- a limitation or toolchain mismatch is hidden behind a green unrelated gate;
- completed edits remain as one oversized commit or an unexplained dirty
  worktree.

## Handoff Record

The final handoff must state what changed, why the owning boundary is correct,
what passed, what was not run, and what risk remains. For retained test or
release evidence, include the artifact path and immutable source commit.

"All tests passed" is insufficient when only a focused selection ran.
"Documentation updated" is insufficient when command examples were not
validated. Precise limits make the result more trustworthy, not less complete.

## Related Guidance

- [Change Validation](change-validation.md)
- [Review Checklist](review-checklist.md)
- [Known Limitations](known-limitations.md)
- [Risk Register](risk-register.md)
