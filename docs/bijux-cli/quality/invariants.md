---
title: Invariants
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Invariants

Use this page when a change feels small in implementation terms but might still
relax a CLI behavior guarantee that scripts, operators, or plugin authors rely
on.

Invariants are the promises `bijux-cli` must preserve across refactors and
feature additions even when internal code structure changes completely.

## Core Invariants

- parser and alias rewrites produce deterministic normalized paths
- help/version short-circuits stay consistent with root command grammar
- structured payload rendering does not mutate semantic meaning
- unknown-route suggestions remain bounded and deterministic
- plugin namespace conflict rules remain strict and explicit

## Code Anchors

- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/routing/model.rs`
- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/routing/registry.rs`
- `crates/bijux-cli/tests/routing/laws/`

## Why These Invariants Matter

| Invariant family | What breaks if it drifts |
| --- | --- |
| parser and route normalization | scripts and documentation no longer name the same command surface |
| output and stream behavior | callers misread success, failure, or machine-readable payloads |
| exit classification | automation reacts incorrectly to user, contract, or internal failures |
| plugin conflict rules | route ownership becomes ambiguous and trust weakens |

## Invariant Rules

- invariant changes require explicit compatibility review
- invariants should be expressed as tests and docs, not prose alone
- do not silently relax invariants to unblock short-term changes

## Reader Shortcut

If a change keeps tests green only by weakening a routing law, output contract,
or conflict rule, the code may compile while the CLI becomes less trustworthy.
That is invariant drift, not harmless cleanup.

## Continue Reading

- [Review Checklist](review-checklist.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Architecture Risks](../architecture/architecture-risks.md)
