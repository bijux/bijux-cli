# Fixture Tooling Coverage Report

## Summary

Fixture tooling coverage is governed across generation, validation, determinism, portability, and lifecycle diagnostics.

## Coverage matrix

| Capability | Coverage surface |
| --- | --- |
| graph fixture generation | `crates/bijux-dag-testkit/src/lib.rs` |
| run fixture generation | `crates/bijux-dag-testkit/src/lib.rs` |
| artifact fixture generation | `crates/bijux-dag-testkit/src/lib.rs` |
| replay fixture generation | `crates/bijux-dag-testkit/src/lib.rs` |
| bundle fixture generation | `crates/bijux-dag-testkit/src/lib.rs` |
| fixture governance reports | `crates/bijux-dev-dag/src/bin/generate_fixture_governance_reports.rs` |
| duplicate helper detection | `crates/bijux-dev-dag/src/bin/generate_duplicate_fixture_loader_report.rs` |
| fixture loader governance contracts | `crates/bijux-dev-dag/tests/fixture_loader_governance_contracts.rs` |

## Completion signals

- governed by `configs/suites/fixture_tooling_governance.json`
- anchored by `crates/bijux-dev-dag/tests/fixture_tooling_completion_contracts.rs`
- corpus coverage tracked in `evidence/cache/fixture_tooling/regression_corpus.json`
