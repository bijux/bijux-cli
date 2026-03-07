# Test coverage map

## Guarantee to test-family mapping

- deterministic scheduling: `crates/bijux-dag-runtime/src/state_machine_tests.rs`
- run summary invariants: `crates/bijux-dag-runtime/src/invariants_tests.rs`
- adapter conformance: `crates/bijux-dag-runtime/src/adapter_contract_tests.rs`
- artifact integrity: `crates/bijux-dag-artifacts/tests/conformance.rs`
- CLI command tree stability: `crates/bijux-dag-app/tests/cli_contract.rs`
- schema contract presence: `configs/schema/fixtures/*` + dev control-plane schema suite

## Missing coverage currently tracked

See `docs/tracking/TEST_DEBT_LEDGER.md`.
