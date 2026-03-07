# Release Policy

## Scope
Defines the minimum evidence required before a release is allowed.

## Release gate requirements
- Contract coverage report has no missing entries.
- Schema coverage report has no missing positive/negative fixture families.
- Docs coverage and taxonomy checks pass.
- Test suites (`checks`, `tests`, `contracts`, `repo`, `docs`) pass.
- E2E matrix report is available and passing.
- Benchmark comparison against previous baseline is within thresholds.
- Resource profile comparison against previous baseline is within accepted thresholds.
- Compatibility matrix for supported schema/graph versions is generated.
- Known limitations are explicitly documented for the release.

## Release blocker classes
- missing contracts
- missing schemas
- missing e2e evidence
- unreviewed performance regression
- undocumented breaking change

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs`
- `bijux-dev-dag release post-release-verify`

## Versioning and change policy
Any relaxation of release requirements is a breaking governance change and requires explicit changelog entry.
