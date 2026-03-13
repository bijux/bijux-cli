# Installation

Use one primary install channel, verify the active runtime immediately, and treat
compatibility alias packages as optional compatibility shims rather than the
default path.

## Verify Any Install

Run these checks after install, upgrade, or uninstall:

```bash
bijux version
bijux cli paths
bijux cli doctor
```

- `bijux version` confirms the command resolves.
- `bijux cli paths` shows the active binary and discovered state paths.
- `bijux cli doctor` reports duplicate installs, path shadowing, stale wrappers,
  and wheel/runtime mismatches.

## Preferred Package Names

- Cargo: `bijux-cli`
- Pip: `bijux-cli`

Compatibility alias packages may still exist for `bijux`. If you intentionally
use them, keep them aligned with the canonical package and verify the resulting
`bijux` binary with the commands above.

## Local Install

### Linux

Cargo:

```bash
cargo install --locked bijux-cli
```

Pip:

```bash
python -m pip install --upgrade bijux-cli
```

Pipx:

```bash
pipx install bijux-cli
```

### macOS

Cargo:

```bash
cargo install --locked bijux-cli
```

Pip:

```bash
python3 -m pip install --upgrade bijux-cli
```

Pipx:

```bash
pipx install bijux-cli
```

### Windows

Cargo:

```powershell
cargo install --locked bijux-cli
```

Pip:

```powershell
py -m pip install --upgrade bijux-cli
```

Pipx:

```powershell
pipx install bijux-cli
```

### Virtual Environments

Use a virtual environment when the Python package should stay isolated from the
system interpreter:

```bash
python -m venv artifacts/python/.venv
source artifacts/python/.venv/bin/activate
python -m pip install --upgrade bijux-cli
```

Validate the install with:

```bash
bijux version
python -m bijux_cli_py version
```

## CI And Containers

Pin the toolchain and install a single channel per image or job.

Cargo-based jobs:

```bash
cargo install --locked bijux-cli
bijux version
```

Python-based jobs:

```bash
python -m pip install --upgrade pip
python -m pip install --upgrade bijux-cli
bijux version
```

Python packaging and release maintainers can build wheels from
`crates/bijux-cli-python` with:

```bash
cd crates/bijux-cli-python
maturin build --release
```

Validate the wrapper against a local Rust binary with:

```bash
PYTHONPATH=python BIJUX_BIN=../../target/debug/bijux python -m pytest -c ../../configs/python/pytest.ini -q tests/python
```

## Multiple Installs

If both Cargo and Pip installs exist:

1. `BIJUX_BIN` wins when set to a valid absolute runtime path.
2. Otherwise, the first `bijux` in `PATH` wins.

Multiple installs are only safe when they expose the same command contract. If
`bijux cli doctor` reports ambiguity or mismatch, remove the extra install or
set `BIJUX_BIN` explicitly for automation.

## Upgrade

Upgrade the channel you actually use, then re-run the verification commands.

Canonical Pip package:

```bash
python -m pip install --upgrade bijux-cli
```

If you intentionally installed the compatibility alias too, keep it aligned:

```bash
python -m pip install --upgrade bijux
```

## Uninstall

Cargo:

```bash
cargo uninstall bijux-cli
```

If you intentionally installed the compatibility alias too:

```bash
cargo uninstall bijux
```

Pip:

```bash
python -m pip uninstall -y bijux-cli
```

If you intentionally installed the compatibility alias too:

```bash
python -m pip uninstall -y bijux
```

After uninstall, run `bijux cli doctor`. If `bijux` still resolves, another
channel still owns the command.

## Python Runtime Notes

- The Python package is a wrapper around the Rust runtime.
- `BIJUX_BIN` can pin the runtime binary when automation must avoid `PATH`
  ambiguity.
- If no usable runtime binary is found, Python entrypoints fail instead of
  silently falling back to a different implementation.
