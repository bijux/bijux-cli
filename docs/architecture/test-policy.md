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

## Review Checklist

Use this checklist during review:

1. Does the change add or update at least one failure-path test?
2. Does it assert output behavior, not just success status?
3. Does it assert exit-code behavior for failures?
4. If stateful, does it include filesystem failure or corruption scenarios?
5. If parser-related, does it include malformed input coverage?
6. If plugin lifecycle-related, does it include rollback coverage?
7. If config mutation-related, does it include corruption-resistance coverage?
8. Are snapshots used only where regression value is clear?
9. Are test names specific about behavior and failure mode?
10. Would the test fail under a realistic regression?

## Priority Inputs

Use generated audit data for current weak spots:

- `artifacts/status/top_20_weakest_tests.json`
- `artifacts/status/top_20_missing_failure_cases.json`
- `artifacts/status/top_20_missing_parity_cases.json`
- `artifacts/status/test_quality_audit.json`
- `artifacts/status/flaky_tests.json`

All flaky tests must be labeled in CI artifacts and tracked explicitly.
