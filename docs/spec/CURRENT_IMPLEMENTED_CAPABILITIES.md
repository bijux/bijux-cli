# Current implemented capabilities

## Scope

This document lists behavior that is implemented in code and protected by passing suites today.

## Implemented and verified

- local DAG execution with deterministic scheduling contracts
- run directory materialization, trace surfaces, and artifact indexing
- policy evaluation, replay semantics, and cache behavior contracts
- import/export, compatibility checks, and operator inspection workflows
- repository governance checks executed by `bijux-dev-dag` foundation suites

## Guard rails

- claims in normative docs must map to at least one code owner path and one test suite.
- unsupported execution modes must be documented under modeled/future boundaries only.

## Related surfaces

- `docs/spec/MODELED_AND_FUTURE_SURFACES.md`
- `docs/spec/SPEC_TO_CODE_AND_TEST_OWNERSHIP.md`
- `docs/reports/foundation/RENOVATION_BURNDOWN_REPORT.md`
