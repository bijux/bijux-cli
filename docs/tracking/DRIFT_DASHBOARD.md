# Drift Dashboard

## Drift status by class

| Drift class | Severity | Status |
| --- | --- | --- |
| docs drift | blocker | monitored |
| schema drift | blocker | monitored |
| contract drift | blocker | monitored |
| cli drift | blocker | monitored |
| test drift | warning | monitored |
| fixture drift | warning | monitored |
| benchmark drift | warning | monitored |
| dependency drift | warning | monitored |

## Drift check ownership
- command tree and CLI docs: `bijux-dev-dag` governance suites
- schema and contract refs: governance suites
- benchmark and release policy alignment: governance suites

## Periodic pruning review
Use [DOCS_PRUNING_CHECKLIST](./DOCS_PRUNING_CHECKLIST.md) and prune dead commands, docs, fixtures, schemas, and policies.
