# Security policy

## Runtime and command hardening

- Commands default to explicit effects and deny-list policy flags.
- Network, environment, and clock effects are rejected at runtime when explicitly denied.
- Shell execution is restricted to the declared effects in each node.

## Supply-chain controls

- `cargo audit` and `cargo public-api` checks are run from `bijux-dev-dag` workflows.
- `.cargo/config.toml` hardens linker behavior on Linux/macOS and enforces local target output.

## Development safety gates

- `verify-tools` and `resolve-check` verify required tooling and dependency graph resolution.
- Dependency policy suites guard forbidden dependencies and legacy workspace references.
