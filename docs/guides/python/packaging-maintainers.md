# Python Packaging For Maintainers

## Build Wheels

Use maturin from `crates/bijux-cli-python`:

```bash
cd crates/bijux-cli-python
maturin build --release
```

## Validate Wrapper Parity

```bash
PYTHONPATH=python BIJUX_BIN=../../target/debug/bijux python -m pytest -q tests/python
```

## Release Notes Requirements

Include:

- extension availability notes
- fallback subprocess behavior notes
- compatibility alias/deprecation notes
