# Nextest configs

- Owner: `platform`
- Purpose: configure Rust test runner behavior for CI and local verification.
- Consumers: `cargo nextest` and Rust verification lanes.
- Update workflow: change the runner config alongside test-lane changes, then rerun Rust verification.
