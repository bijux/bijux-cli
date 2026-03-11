# {{cookiecutter.project_name}}

A cookiecutter template for a Bijux plugin with a Rust codebase and a Python host shim.

## Quick start

```bash
# Scaffold from your workspace root
mkdir -p ./tmp && cd ./tmp
bijux plugins scaffold my_rust_plugin --template=../templates/plugins-rs --force

# Install and inspect
cd ..
bijux plugins install ./tmp/my_rust_plugin --force
bijux plugins info my_rust_plugin
bijux plugins check my_rust_plugin
```

## Generated structure

- `plugin.json`: plugin metadata consumed by Bijux plugin discovery.
- `plugin.py`: Python runtime shim used by the current plugin host.
- `Cargo.toml` and `src/lib.rs`: Rust crate scaffold for native logic.

## Development notes

- Implement your core logic in Rust and expose it through your preferred bridge.
- Keep `plugin.py` as the stable host entry for current plugin lifecycle commands.
- Update `bijux_cli_version` in `plugin.json` when you bump host compatibility.
