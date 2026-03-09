# Python Packaging For Maintainers

## Build Wheels

Use maturin from `packages/bijux-cli-py`:

```bash
cd packages/bijux-cli-py
maturin build --release
```

## Validate Wrapper Parity

```bash
PYTHONPATH=python BIJUX_BIN=../../target/debug/bijux-rs python -m pytest -q tests
```

## Release Notes Requirements

Include:

- extension availability notes
- fallback subprocess behavior notes
- compatibility alias/deprecation notes
