---
title: Test Philosophy
audience: maintainer
type: specification
status: canonical
owner: bijux-core-quality
last_reviewed: 2026-07-19
---

# Test Philosophy

Tests are executable trust claims. A test is valuable when its setup reaches
owned behavior, its assertions distinguish correct from incorrect semantics,
and its failure identifies a product or governance contract that no longer
holds. Test count and line coverage are supporting measurements, not proof by
themselves.

## Evidence Standard

Every supported claim should connect:

1. an owning package or product boundary;
2. a contract that states expected behavior and refusal;
3. an executable test that reaches that behavior;
4. retained evidence when review depends on generated or external results.

The narrowest proven claim wins. A unit test does not prove an installed
binary, a mocked backend does not prove a live backend, and a source-tree test
does not prove a published package.

## Test Classes

| Class | Purpose | Required quality |
| --- | --- | --- |
| unit | local transformation or invariant | isolates the owned behavior and asserts semantics |
| contract | stable boundary between modules, crates, schemas, or commands | exercises both acceptance and refusal |
| integration | collaboration across owned boundaries | uses real collaborators unless the boundary itself is the subject |
| end-to-end | supported operator workflow | enters through a public surface and inspects externally visible results |
| adversarial | malformed, hostile, corrupt, or boundary-stressing input | proves bounded failure without weakening validation |
| property or fuzz | behavior over a generated input space | retains or reports a reproducible failing case |
| platform | behavior dependent on an operating system or external backend | records the real platform and capability |
| release | installable artifact behavior | validates the built or packaged artifact, not only workspace source |

Names describe the behavior under proof. A test named only after a bug number,
delivery sequence, or broad subsystem does not communicate durable intent.

## Semantic Assertions

Prefer assertions over typed values, identities, state transitions, error
classes, and retained records. Text snapshots are appropriate only when exact
human output is the contract. Do not use broad snapshots to conceal which
fields matter.

Normalization may remove only fields declared non-semantic by the owning
contract. It cannot erase ordering, status, identity, provenance, timestamps
that affect behavior, or unexpected fields merely to stabilize output.

## Doubles And Fixtures

A fake is honest when the test is proving how the subject interacts with that
boundary. It is dishonest when it repeats the implementation or is used to
claim behavior of an external system.

Shared fixtures:

- have an owner and a named semantic purpose;
- remain deterministic for supplied inputs;
- fail when required data is absent or malformed;
- do not silently acquire new defaults when production contracts grow;
- distinguish synthetic examples from release evidence.

## Failure And Recovery

Critical behavior includes failure, refusal, and recovery paths. Success-only
coverage is insufficient for:

- graph validation and identity;
- scheduler and state-machine transitions;
- cache integrity and replay eligibility;
- artifact corruption and unsafe paths;
- policy and security boundaries;
- package installation and compatibility.

Retries do not turn a flaky test into trusted evidence. Nondeterminism must be
removed, controlled by an explicit seed/clock, or classified with an owned
remediation record.

## Fast, Slow, And Ignored Work

Execution lane is independent from importance. Fast tests provide routine
feedback. Slow tests remain governed and run in their declared lane. Ignored
tests are not passing evidence and require an explicit reason and execution
path.

Filtering, advisory mode, platform exclusion, retries, and skipped work must be
visible in the result. A partial run cannot be summarized as the full suite.

## Review Questions

A reviewer should be able to answer:

- Which behavior would regress if this test were removed?
- Can the test fail when the implementation is wrong?
- Does it call the owned behavior rather than duplicate it?
- Are acceptance and important refusal paths represented?
- Are mocks, fixtures, platforms, filters, and ignored work disclosed?
- Does the test prove only the claim stated by its name and evidence?
- Is failure reproducible from the retained command and inputs?

`TEST_TRUST_CONTRACT.md` governs the required trust surfaces.
`TEST_TRUST_LEDGER.md` governs machine-readable classification and coverage
families.
