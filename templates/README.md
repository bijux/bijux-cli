# Plugin Templates

Repository-supported plugin scaffolds live under this directory.

- `plugins-py`: Python-first plugin scaffold for `bijux plugins scaffold --template ...`.
- `plugins-rs`: Rust-backed scaffold with a Python host shim and Rust crate baseline.

## Usage

```bash
# Python plugin template
bijux plugins scaffold my_python_plugin --template ./templates/plugins-py --force

# Rust-backed plugin template
bijux plugins scaffold my_rust_plugin --template ./templates/plugins-rs --force
```

Both templates are cookiecutter-compatible and produce a plugin directory with valid `plugin.json` metadata.
