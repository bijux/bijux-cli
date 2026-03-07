# Concurrency Flake Ledger

| id | first_seen | area | symptom | status | owner | notes |
| --- | --- | --- | --- | --- | --- | --- |
| placeholder-0001 | 2026-03-07 | scheduler | no known flakes yet | open | runtime | keep ledger even when empty |

## Policy

- Record every nondeterministic failure in this ledger before closing work.
- Include reproduction command and artifact path in `notes`.
- Do not delete entries; close with final remediation evidence.
