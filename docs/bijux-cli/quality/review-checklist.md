---
title: Review Checklist
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Review Checklist

Use this checklist for every `bijux-cli` change that touches runtime behavior,
contracts, or documentation.

The goal is not to mechanically clear boxes. The goal is to stop reviewers from
approving a change whose behavior, evidence, and written contract do not agree.

## Checklist

1. Scope: does the change stay inside CLI ownership, or is it actually a DAG or repo-level change?
2. Contracts: do command grammar, payloads, output streams, or exit semantics change?
3. Tests: are the owning routing, integration, or architecture lanes updated and sufficient?
4. Docs: do the affected handbook pages explain the new behavior in reader language?
5. Compatibility: would a script, plugin author, or operator need an explicit warning?
6. Risk: did the change increase trust, plugin, or persistence risk without naming it?
7. Gates: do the relevant docs and test surfaces pass on the actual change set?

## What Reviewers Should Refuse To Merge

| Smell | Why it is a blocker |
| --- | --- |
| tests pass but docs still describe old behavior | readers and automation receive conflicting truth |
| docs changed without code or evidence for the claim | the page becomes aspirational instead of factual |
| compatibility impact is implied but not written down | downstream callers absorb breakage silently |
| a change crosses ownership boundaries without saying so | the wrong surface gets reviewed and trusted |

## Documentation Shape Guardrails

- `docs/bijux-cli/` contains exactly 6 directories and 51 files
- exactly 5 section directories under the package root
- each section contains exactly 10 pages

## Code Anchors

- `makes/docs.mk`
- `docs/bijux-cli/`
- `crates/bijux-cli/tests/`

## Reader Shortcut

If the review conversation depends on "everyone knows what this really means,"
the checklist has already found a gap. The change is only ready when a new
reviewer can infer the same conclusion from code, tests, and docs alone.

## Continue Reading

- [Documentation Standards](documentation-standards.md)
- [Definition of Done](definition-of-done.md)
- [Change Validation](change-validation.md)
