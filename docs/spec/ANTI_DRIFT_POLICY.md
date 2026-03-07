# Anti-Drift Policy

## Drift classes
- docs drift
- schema drift
- contract drift
- cli drift
- test drift
- fixture drift
- benchmark drift
- dependency drift

## Drift blocker severities
- blocker: fails governance checks
- warning: reported but does not fail governance checks

## Required same-change alignment rule
Any change to a normative surface must update:
- owning contract doc
- tests or fixtures proving behavior
- user-facing docs that describe behavior

## Required checks
- command docs align with command tree
- JSON output docs align with schemas
- invariants docs align with invariant registry
- contract references align with tests
- docs examples align with executable fixtures
- crate graph docs align with cargo metadata evidence
- version support docs align with compatibility fixtures
- benchmark docs align with scenario definitions
- release policy docs align with release-readiness verification suite

## Repository trust summary
`bijux-dev-dag repo-trust-summary` is the control-plane command for domain trust status.
