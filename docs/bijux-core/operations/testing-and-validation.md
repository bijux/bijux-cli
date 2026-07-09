---
title: Testing and Validation
audience: mixed
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Testing and Validation

Validation is the point where `bijux-core` decides whether a change is merely
edited or actually supported. A passing narrow test can show that one crate is
healthy. It does not automatically prove that shared contracts, generated
references, released surfaces, or retained evidence still match the repository
story.

The job is not to run the most commands. The job is to run the right commands
for the surface that changed.

## Start With The Surface, Not The Tool

Choose validation by asking what could have drifted:

| Changed surface | What needs proof |
| --- | --- |
| one implementation detail inside a crate | the owning crate still behaves as intended |
| public command output or schema | compatibility, snapshots, and generated references still match |
| DAG runtime or retained artifact behavior | runtime suites and retained evidence layouts still agree |
| root docs or navigation | the handbook still builds and routes readers correctly |
| root workflow, release, or contract behavior | repository-level gates still enforce the published boundary |

## Useful Root Entry Points

These root commands are the standard starting points when the change crosses a
repository boundary:

```bash
make test
make dag-test
make docs-check
```

They are entrypoints, not an excuse to skip ownership analysis. If the change
is narrower, a focused crate or suite may be the better first proof.

## Validation By Change Type

### Crate-local implementation work

Start with the owning crate's tests when the change clearly stays inside one
implementation boundary and does not alter a shared contract or public output.

### Public output, schema, or snapshot work

Add the checks that prove visible behavior, machine-readable envelopes,
generated references, or golden outputs still match the documented contract.

### DAG runtime and retained artifact work

Expect cross-crate proof more often here. Runtime behavior, manifests, replay,
and retained run directories can drift apart if only one layer is exercised.

### Docs and navigation work

When a public or maintainer-facing explanation changed, run the docs build or
the narrower docs checks that prove the site still renders and routes cleanly.

### Root automation or release work

Use the repository gates that prove workflow, release, and contract surfaces
still behave honestly above any one crate.

## What Good Evidence Looks Like

A strong validation set is:

- tied to the owning surface
- broad enough to cover the changed meaning
- narrow enough that a reviewer can understand why it was chosen
- explicit about any skipped gate and why it was skipped

## Under-Validation Mistakes

- relying on unit tests after changing public command output
- updating retained artifact behavior without checking golden or integration
  evidence
- changing docs claims without confirming the site still builds
- touching release or workflow surfaces without root-level proof

## Over-Validation Mistakes

- running the heaviest repository gate when a focused suite already proves the
  change
- repeating large root suites after a bounded prose-only clarification
- using "I ran everything" as a substitute for understanding ownership

## Working Rule

Validation should prove the changed surface honestly and no broader than
necessary.

## Next Reads

- [Review Expectations](review-expectations.md)
- [Change Management](change-management.md)
- [Core Testing and Validation](../governance/testing-and-validation.md)
