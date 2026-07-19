---
title: Drift Dashboard
audience: maintainers
type: report
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-06
---

# Drift Dashboard

This dashboard tracks the repository drift classes that the governance suite
expects maintainers to watch before merging release-facing changes.

| Drift class | Severity | Primary check |
| --- | --- | --- |
| docs drift | blocker | `repo-docs` |
| schema drift | blocker | `docs-schema-ref` |
| contract drift | blocker | `docs-contract-ref` |
| cli drift | blocker | `cli-freeze` |
| test drift | warning | `contract-test-links` |
| fixture drift | warning | `docs-coverage` |
| benchmark drift | warning | `performance-claims` |
| dependency drift | warning | `dependency-policy` |

## Maintenance rule

When a new drift class becomes release-relevant, add it here and link it to
its enforcing repo check in the same change.
