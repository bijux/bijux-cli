# Acceptance gates for reliable delivery

## Required gates

- Formatting and lint checks pass.
- Contract test suites pass for core, runtime, and CLI surfaces.
- Compatibility fixtures show no unapproved drift.
- Public API checks show no unapproved drift.
- Dependency audit has no blocking vulnerabilities.
- Release verification sequence passes.

## Backend production-capability gates

A backend is production-capable only when all of the following are true:

- deterministic replay behavior is validated against conformance fixtures
- artifact integrity and transport checks pass
- policy enforcement behavior is verifiable and audited
- observability coverage includes events, metrics, and timeline exports
- backend request/completion serialization contracts are stable

## Decision rule

A release candidate is accepted only when all required gates pass without bypass.

## Evidence

Store machine-readable reports under `artifacts/reports` and publish in CI artifacts.
