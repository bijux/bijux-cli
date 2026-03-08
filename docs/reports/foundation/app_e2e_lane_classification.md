# App E2E Lane Classification

generated_from: `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs` and lane rationale policy

## Classification Summary

- fast-core promoted scenarios: `app_smoke_routed_workflows_contract`
- slow-extended scenarios: `e2e_integration_scenarios` ignored tests and heavy import/export contracts

Rationale source of truth:

- `configs/policy/app_e2e_lane_rationale.json`

Suite split:

- fast core: `configs/suites/app_e2e_fast_core.json`
- slow extended: `configs/suites/app_e2e_slow_extended.json`
