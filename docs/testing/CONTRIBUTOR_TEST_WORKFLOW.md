# Contributor Test Workflow

generated_from: `FAST_FULL_TEST_RULES.md`

## Minimal Local Loop

1. run `make test`
2. fix failures
3. run focused crate tests for touched areas

## Pre-Release Loop

1. run `make test-release`
2. verify generated reports in `docs/reports/foundation`
3. verify release gates pass in `crates/bijux-dev-dag/tests`
