# Development workflow tests

This crate owns repository-level checks that coordinate cross-crate policy.

- `no_cli_in_runtime.rs`: ensures runtime does not depend on CLI/app crates.
- `no_runtime_in_core.rs`: ensures core does not depend on runtime.
- `no_core_io.rs`: ensures core stays I/O-free.
