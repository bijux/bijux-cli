---
title: Documentation Standards
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Documentation Standards

Documentation in `bijux-core` is expected to help a reader reach the truth
faster, not just satisfy a formatting rule. These standards exist so pages
remain accurate, navigable, and specific enough to survive release pressure
and critical review.

## What Strong Repository Documentation Looks Like

A good page in this repository does four things at once:

- it tells the reader what surface the page owns
- it distinguishes current supported behavior from internal, experimental, or
  future work
- it points the reader toward the next concrete source of truth
- it stays close enough to code, contracts, and tests that drift is visible

## Standards That Matter Most

### Write for the reader's question

Start with the practical question the page answers. Pages should not open by
describing documentation process unless the process itself is the subject.

### Name the owning surface clearly

Readers should be able to tell whether the page is about the CLI runtime, the
DAG stack, the maintainer control plane, or a shared repository boundary.

### State capability boundaries directly

If something is stable, say so. If it is experimental, simulated, internal, or
future, say that just as plainly. Avoid language that lets aspirational work
sound shipped by accident.

### Use proof-bearing references

Behavior claims should route readers toward the code, contract, generated
reference, or test suite that would expose drift.

### Prefer realistic examples

Commands, paths, and examples should reflect how the repository actually works
today. Placeholder-style examples are acceptable only when a page is about a
pattern rather than a specific supported workflow.

## Language Rules

- prefer direct declarative language over hedging
- use ownership verbs such as `owns`, `publishes`, `enforces`, `proves`, and
  `does not support`
- avoid vague filler such as "basically," "generally," or "kind of"
- when uncertainty is real, name the uncertainty and the validating surface

## Structural Rules

Every handbook page should make the following easy to find:

- what the page is about
- which audience it is for
- where the authoritative next read lives

Canonical frontmatter remains required:

- `title`
- `audience`
- `type`
- `status`
- `owner`
- `last_reviewed`

The exact section order can vary when the page reads better that way. The
important rule is that structure should help orientation, not become a ritual.

## Common Documentation Failures

- process-oriented opening paragraphs on pages readers expect to be product or
  repository guidance
- claims that outgrow the real release boundary
- reports that imply support instead of summarizing evidence
- pages that name paths but never explain why the reader should care
- examples that look synthetic enough to undermine trust

## Repository Anchors

These roots and files shape most documentation decisions:

- `mkdocs.yml` for published navigation
- `mkdocs.shared.yml` for shared site behavior
- `makes/docs.mk` for docs build and validation entrypoints
- `docs/index.md` for top-level site routing

## Next Reads

- [Decision Record Policy](decision-record-policy.md)
- [Risk and Exceptions](risk-and-exceptions.md)
- [Maintainer Documentation Standard](../../bijux-dev/governance/documentation-standard.md)
