---
title: Documentation Governance Alignment
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Documentation Governance Alignment

This page explains how repository documentation stays aligned with release
truth, executable proof, and package ownership. The goal is not to make every
page sound the same. The goal is to make sure the repository does not publish
one story in prose while code, contracts, and tests prove another.

## The Alignment Problem

`bijux-core` publishes several overlapping surfaces at once:

- product handbooks
- crate READMEs and package pages
- release-boundary and compatibility contracts
- maintainer reports and evidence summaries

Those surfaces do not all serve the same audience, but they still need to agree
on what exists, what is public, what is experimental, and what is only an
internal or simulated lane.

## What Alignment Means Here

Documentation is aligned when all of the following are true:

- the owning handbook matches the owning package or contract surface
- release framing does not promise more than the release boundary allows
- reports summarize evidence without silently creating new product claims
- capability language distinguishes real shipped behavior from modeled,
  internal, or future work

## The Main Authority Chain

When two documentation surfaces disagree, resolve the conflict in this order:

1. executable behavior and enforcing tests
2. machine-readable contracts and release truth tables
3. canonical handbook pages and package documentation
4. reports, summaries, and roadmap-style material

That order keeps explanatory material from outranking the surfaces that the
repository actually validates.

## Pages That Carry The Core Boundary

| Question | Canonical page |
| --- | --- |
| What does the repository publish and how is it split? | [Platform Overview](../foundation/platform-overview.md) |
| What is the stable `bijux-dag` product boundary today? | [Release Boundary](../../bijux-dag/foundation/release-boundary.md) |
| How must docs claims stay tied to code and tests? | [Spec To Code And Test Ownership](spec-to-code-and-test-ownership.md) |

## What This Page Protects Against

- handbook prose outrunning the real release boundary
- reports being read as product promises
- package pages and root pages drifting apart on public-versus-private status
- "future" or "modeled" work being described as if it already ships

## A Useful Review Habit

When you touch a docs page that makes a capability claim, check three things
before calling it done:

1. Which package or contract owns the claim?
2. Which release boundary allows the claim to be stated publicly?
3. Which test, suite, or generated reference would expose drift if the claim
   stops being true?

If those answers are weak, the page is probably still relying on inference
instead of repository truth.

## Next Reads

- [Documentation Standards](documentation-standards.md)
- [Change Management](../operations/change-management.md)
- [Testing and Validation](../operations/testing-and-validation.md)
