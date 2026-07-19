---
title: Change Validation
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Change Validation

This page is for the change author. Its purpose is to select evidence from the
behavioral boundary that changed, not to prescribe one expensive command for
every edit.

## Classify Before Running

| Changed boundary | Minimum focused evidence | Widen when |
| --- | --- | --- |
| graph model, parsing, validation, identity, or planning | owning `bijux-dag-core` test binary and dependency-direction contracts | serialized graph, public import, or plan compatibility changes |
| retained models, storage, integrity, lineage, or lifecycle | owning `bijux-dag-artifacts` tests plus affected round trip | runtime or replay consumes the changed evidence |
| scheduler, adapter, policy, cache, or replay | focused `bijux-dag-runtime` contracts | app output, retained schema, or backend equivalence changes |
| command orchestration or response model | focused `bijux-dag-app` route and schema contracts | visible command behavior or CLI snapshots change |
| process entrypoint, help, or exit mapping | `bijux-dag-cli` contract and smoke tests | command inventory or release lane changes |
| package dependency or publication metadata | dependency, package-boundary, package-list, and publish dry-run checks | public crate graph or MSRV changes |
| handbook or generated reference | documentation source/link contracts and `make docs-check` | example behavior or public command claims also change |

`configs/dag/release/change_impact_commands.json` provides package-level
starting commands. It is not a proof that cross-package consequences were
considered.

## Compatibility Questions

Before calling the change internal, determine whether it alters:

- accepted or rejected graph input;
- canonical bytes, fingerprints, cache keys, or node identity;
- retained manifest, trace, index, proof, or bundle shape;
- retry, timeout, cancellation, branch, or terminal-state semantics;
- machine-readable response fields, error codes, or process exits;
- stable Rust imports or visible command discovery;
- release classification, package metadata, or publish order.

If any answer is yes, identify the compatibility authority and include an old
fixture, migration, explicit refusal, or release note as appropriate. A new
test that asserts only the new behavior does not establish compatibility.

## Evidence Sequence

1. Run the smallest failing or proving test at the owning boundary.
2. Run adjacent contract tests where values cross into another crate.
3. Regenerate governed references with their named producer.
4. Inspect generated diffs before staging them.
5. Run the narrow repository lane that includes the changed boundary.
6. Record exact commands, final status, and intentionally omitted lanes.

Do not regenerate a snapshot or baseline until the behavioral difference is
understood. Generated agreement can preserve a regression as easily as it can
record an intentional contract change.

## Failure Discipline

When an existing test fails, decide whether implementation, fixture, generated
evidence, or the asserted contract is wrong. Check adjacent tests before
changing shared behavior. Do not weaken assertions, move tests into a slow
lane, add retries, or convert required checks to advisory status merely to
make the change pass.

Preserve the focused failure and terminal summary under `artifacts/` when they
are needed for review.

## Author Handoff

The author should provide:

- behavioral summary and owning package;
- compatibility conclusion;
- exact focused and widened commands;
- passed, failed, skipped, slow, or omitted scope;
- governed outputs changed and their producer;
- remaining limitation or risk.

The reviewer uses [Review Checklist](review-checklist.md) to verify those
claims independently. Completion is governed by
[Definition Of Done](definition-of-done.md).
