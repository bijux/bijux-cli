# Maintainer Milestone Checklist

Use this checklist for every milestone claim.

## Required Outcome Shape
1. `done`: what is complete with artifact evidence.
2. `left`: what is still missing with blocker and owner.
3. `blocked/deferred`: what is intentionally deferred and why.

## Required Evidence
- `artifacts/status/what_is_done.json`
- `artifacts/status/what_is_left.json`
- `artifacts/status/what_is_deferred.json`
- `artifacts/status/what_is_partial.json`
- `artifacts/status/what_is_unproven.json`

## Claim Discipline
1. No status language without generated evidence.
2. Reviewers reject hype phrasing not backed by artifacts or tests.
3. Check `bijux dev cli status --format json` first before any milestone claim.

## Planning Discipline
Next backlog must come from generated status data only:
- `artifacts/status/next_200_todos.json`
