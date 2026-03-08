# App Hot-Path Fast Suite

generated_from: `crates/bijux-dag-app/tests`

## Suite Members

- `crates/bijux-dag-app/tests/help_surface_contracts.rs`
- `crates/bijux-dag-app/tests/command_surface_routing_contracts.rs`
- `crates/bijux-dag-app/tests/operator_malformed_input_no_panic_contracts.rs`
- `crates/bijux-dag-app/tests/config_effective_command_contract.rs`
- `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`

## Promotion Criteria

- deterministic execution
- no network dependency
- no external backend binary requirement
- stable under `cargo test -p bijux-dag-app`
