# App Smoke Release Coverage Report

generated_from: `crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs`

Smoke coverage checklist:

- `validate → plan → run → inspect/status → replay → diff` : covered
- `artifact hash → inspect → trace` : covered
- `export → import verify-only → fsck` : covered
- `history → show → summary → timeline` : covered
- `prove → verify` : covered
- `semantic-portability` and `capabilities --backend` : covered

Fixture used for canonical workflow:

- `crates/bijux-dag-app/tests/fixtures/git_for_computation_graphs_workflow.json`
