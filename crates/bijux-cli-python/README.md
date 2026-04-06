# bijux-cli Python Package

`bijux-cli` is the Python distribution for installing and launching the Bijux
command runtime.

This package provides:

- the `bijux` console entrypoint,
- a native Rust bridge module (`bijux_cli_py._native`) when available,
- a Python facade fallback for portability and compatibility checks.

## What This Package Is

- A packaging and bridge layer for the `bijux` runtime.
- The canonical PyPI surface for `bijux-cli`.
- A compatibility boundary between Python callers and the Rust runtime.

## What This Package Is Not

- It does not define independent runtime semantics.
- It does not publish maintainer control-plane commands.
- It does not replace repository-level governance docs.

## Quick Usage

```bash
python -m pip install bijux-cli
bijux --help
python -m bijux_cli_py --help
```

## Source of Truth

- Runtime crate: `crates/bijux-cli`
- Python bridge crate: `crates/bijux-cli-python`
- Package changelog: `crates/bijux-cli-python/CHANGELOG.md`
- Repository handbook: `docs/bijux-cli/`

## License

Apache-2.0 (`LICENSE`).
