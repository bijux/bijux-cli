# ADR: Dashboards and Readiness Metrics

## Status

Accepted

## Context

System quality signals exist across many reports, but operators and maintainers need a stable, consolidated readiness surface that is test-enforced.

## Decision

1. Maintain explicit dashboard pages for each major reliability and compatibility dimension.
2. Keep an overall readiness dashboard as a stable entrypoint.
3. Enforce dashboard presence and mapping through contract tests.
4. Require a verification suite that combines dashboard contracts with key subsystem contracts.

## Consequences

- Dashboard drift becomes visible and merge-blocking.
- Readiness review uses a single index instead of ad-hoc report discovery.
- Engineering ownership of readiness metrics stays explicit over time.

## Enforcement

- `crates/bijux-dev-dag/tests/system_readiness_dashboard_contracts.rs`
- `configs/suites/system_readiness_dashboards_verification.json`
