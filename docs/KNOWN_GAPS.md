# Status And Gaps

Use tests and live maintainer commands as the source of truth for current
status and remaining gaps. Generated artifacts are disposable outputs, not repo
inputs.

## Current Inputs

- `cargo test --workspace`
- `python3 -m pytest crates/bijux-cli-python/tests/python`
- `bijux dev cli status --format json --no-pretty`
- `bijux dev cli parity --format json --no-pretty`

## Claim Discipline

- Do not describe work as complete unless the supporting test or live maintainer
  command is current and green.
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

Review these commands before publishing status:

- `bijux dev cli status --format json --no-pretty`
- `bijux dev cli parity --format json --no-pretty`
- `bijux dev cli docs-audit --format json --no-pretty`

For configuration-specific parity gaps and deferred changes, use the
[Configuration and state architecture](10-architecture/configuration-and-state.md)
together with generated parity evidence rather than older hand-maintained gap
lists.
