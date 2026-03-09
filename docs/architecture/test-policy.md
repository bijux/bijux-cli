# Test Policy

## Principle
Tests must bite. A test that only proves the default happy path without guarding a realistic failure mode is incomplete.

## Required Coverage Per New Command
- at least one failure-path test
- at least one output regression test
- at least one exit-code test

## Required Coverage Per Feature Type
- stateful commands: at least one filesystem-failure test
- parser features: at least one malformed-input test
- plugin lifecycle features: at least one rollback test
- config mutation features: at least one corruption-resistance test
- output changes: snapshot or diff coverage is required
- install changes: ambiguity or path-failure coverage is required

## Quality Rules
- No vanity test counts in release or status claims.
- No vanity test growth.
- Coverage claims must be paired with failure-scenario evidence.
- Flaky tests must be explicitly labeled and tracked as debt.
- Tests explicitly tagged as filler are rejected in CI.

## Truth Sources
- `artifacts/status/test_quality_audit.json`
- `artifacts/status/flaky_tests.json`
- `artifacts/parity/rust_python_parity_report.json`
