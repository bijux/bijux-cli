# Mounted Python Apps

`bijux-cli-python` ships `bijux_cli_py.app_sdk` so Python packages can mount
cleanly under the root `bijux` command surface.

## What the helper owns

- product-mount manifest construction for Python apps
- host compatibility reporting against the active `bijux` runtime version
- JSON success and failure envelopes that match the Rust runtime contract
- stdout/stderr discipline for callable-style mounted commands

It does not own discovery policy. The Rust root runtime still decides where
mount manifests are loaded from and which descriptor wins.

## Minimal package layout

```text
sample-app/
  pyproject.toml
  sample_app/
    __init__.py
    cli.py
  .bijux/
    apps/
      sample.mount.json
```

`sample_app/cli.py`:

```python
from bijux_cli_py.app_sdk import run_json_app, success


def main(argv: list[str]):
    return success({"argv": argv}, command=["sample"])


if __name__ == "__main__":
    run_json_app(main)
```

Manifest generation:

```python
from bijux_cli_py.app_sdk import CompatibilityWindow, build_python_mount_manifest

manifest = build_python_mount_manifest(
    namespace="sample",
    display_name="Sample App",
    module="sample_app.cli",
    function="main",
    summary="Sample mounted Python app",
    compatibility=CompatibilityWindow(min_cli_version="0.4.0"),
)
```

## Descriptor contract

Python mounted apps use `entrypoint.kind = "python_module"` and may carry:

- `command`: interpreter command hint
- `module`: importable module path
- `function`: optional callable inside that module

When `function` is present, `bijux` invokes the mounted app through a callable
runner. When it is absent, `bijux` falls back to `python -m <module>`.

## Interpreter discovery

The root runtime resolves Python in this order:

1. active virtual environment
2. project-local `.venv`
3. `BIJUX_PYTHON_BIN`
4. `python3` / `python` from `PATH`

Use `bijux apps doctor <namespace>` to inspect the chosen interpreter, import
status, package metadata, and callable probe results.

## Output contract

Mounted commands should keep structured result JSON on stdout and direct
incidental logs to stderr. `run_json_app(...)` enforces that split by capturing
stdout from the app callable and replaying it on stderr before emitting the
final JSON envelope on stdout.

## Verification surfaces

- Rust schema and routing contracts: `cargo test -p bijux-cli --tests`
- Python helper parity/contracts:
  `PYTHONPATH=crates/bijux-cli-python/python BIJUX_PY_ALLOW_NATIVE_OSERROR_FALLBACK=1 python3 -m pytest crates/bijux-cli-python/tests/python/test_app_sdk.py`
