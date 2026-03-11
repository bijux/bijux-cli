# Python Bindings

## Build Stack

- Primary binding path: `PyO3 + maturin`
- Python package root: `crates/bijux-cli-python`
- Rust extension source: `crates/bijux-cli-python`

## Package Naming

- Canonical distribution: `bijux-cli`
- Compatibility/meta distribution reserved: `bijux`

## Entrypoint Ownership

The Python distribution installs a `bijux` console script that delegates to the Rust-backed runtime facade and must not diverge from canonical CLI behavior.

## Wheel Strategy

Wheels are produced through maturin and target Linux/macOS/Windows build matrices with platform-specific artifacts.

## Exposed Python Facade APIs

- `version()`
- `command_tree_introspection()`
- `execution_facade(argv)`
- `output_envelope_model()`
- `error_to_exception(payload)`
- `config_resolution_helpers(home_dir)`
- `plugin_registry_inspection(registry_file)`
- `install_path_helpers(home_dir)`
