---
title: Known Limitations
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Known Limitations

Use this page when a maintainer result is technically green but still needs
human judgment, extra runtime, or environment awareness before you trust it.

Known limitations are not an apology for weak tooling. They are the boundary
between what the repository can prove today and what still requires operator
care.

## Limitation Categories

- long-running suites with high local runtime cost
- partial automation for edge-case release recovery
- diagnostics that still require manual interpretation in rare failures
- environment-dependent behavior outside supported baselines

## Limitation Rules

- document impact and mitigation for each limitation
- remove obsolete limitations when capability matures
- do not label unresolved limitations as complete coverage

## How To Read A Limitation

| Question | Why it matters |
| --- | --- |
| what does the limitation block? | tells you whether release, recovery, or diagnosis is affected |
| what mitigation exists today? | gives the maintainer a usable fallback instead of vague caution |
| when should the limitation be removed? | prevents stale caveats from becoming permanent folklore |

## Reader Shortcut

A limitation is honest documentation only when it names the operational cost.
If the page says something is limited but does not explain the practical
consequence, it is still hiding the real problem.

## Code Anchors

- `crates/bijux-dev/tests/`
- `crates/bijux-dev/src/suites/`
- `makes/`

## Continue Reading

- [Test Policy](test-policy.md)
- [Incident Response](../operations/incident-response.md)
- [Core Architecture Risks](../../bijux-core/architecture/architecture-risks.md)
