# Status And Gaps

Use generated artifacts as the source of truth for current status and remaining
gaps.

## Current Inputs

- `artifacts/parity/command_parity_matrix.json`
- `artifacts/status/status_known_parity_gaps.json`
- `artifacts/status/runtime_unity_report.json`
- `artifacts/status/docs_audit.json`
- `artifacts/status/test_quality_audit.json`

## Claim Discipline

- Do not describe work as complete unless the supporting artifact exists and is current.
- Treat missing evidence as an open gap, not an implicit success.
- Review `bijux dev cli status --format json` before making maintainer status claims.

## Stability Rules

Treat a surface as stable only when:

- command identity and route behavior are covered by current parity or contract tests
- stderr/stdout and exit-code behavior are covered where the surface is exposed
- install, plugin, or state diagnostics do not report unresolved blockers for that area

Treat a surface as still risky when:

- parity rows are `partial` or `missing`
- plugin lifecycle write paths are still under hardening
- documentation or tests still mark the area as intentionally different or unresolved

## What Not To Claim

- no completion parity claim while parity evidence is still partial or missing
- no runtime convergence claim without runtime identity and install diagnostics evidence
- no plugin stability claim without registry, lifecycle, and health evidence

## Maintainer Checklist

For release or review summaries, keep the outcome shape explicit:

1. `done`
2. `left`
3. `blocked` or `deferred`

Review these artifacts before publishing status:

- `artifacts/status/what_is_done.json`
- `artifacts/status/what_is_left.json`
- `artifacts/status/what_is_partial.json`
- `artifacts/status/what_is_deferred.json`
- `artifacts/status/what_is_unproven.json`
- `artifacts/status/next_200_priorities.json`

For configuration-specific parity gaps and deferred changes, use
`docs/architecture/config-parity-report.md` together with the generated parity
artifacts rather than older hand-maintained gap lists.
