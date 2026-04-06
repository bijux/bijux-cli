# Installation And Recovery

## Goal

Use one install channel at a time, understand the supported install shapes, and
recover cleanly when path ambiguity or wrapper drift appears.

## Supported Install Channels

Canonical package names:

- Cargo: `bijux-cli`
- Pip: `bijux-cli`
- Pipx: `bijux-cli`

## Local Install Commands

### Linux and macOS

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

### Unsupported platforms

Windows is not a supported platform today. Do not treat an installation that
appears to work on Windows as part of the supported contract.

That boundary is intentional and current. The Rust runtime can already handle
some Windows-specific concerns such as `.exe` discovery and state-path
resolution, but the built-in Rust plugin scaffold and completion integration
still assume Linux/macOS host conventions.

## Virtual Environments

Use a virtual environment when the Python package should stay isolated from the
system interpreter. For a local checkout, prefer a normal project-local `.venv`
rather than a repo-specific artifact path:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade bijux-cli
```

Validate the install with:

```bash
bijux version
python -m bijux_cli_py version
```

## CI And Container Use

Install one channel per image or job.

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

Python packaging maintainers can build wheels from
`crates/bijux-cli-python` with:

```bash
cd crates/bijux-cli-python
maturin build --release
```

## Upgrade And Uninstall

Upgrade the channel you actually use, then rerun:

```bash
bijux version
bijux cli paths
bijux doctor
```

Cargo uninstall:

```bash
cargo uninstall bijux-cli
```

Pip uninstall:

```bash
python -m pip uninstall -y bijux-cli
```

If `bijux` still resolves after uninstall, another channel still owns the
command.

## Recovery Rules

If both Cargo and Pip installs exist:

1. `BIJUX_BIN` wins when set to a valid absolute runtime path
2. otherwise, the first `bijux` in `PATH` wins

If `bijux doctor` reports ambiguity or mismatch:

- remove the extra install
- or set `BIJUX_BIN` explicitly for automation

## Honest Limit

An install that "works on my machine" but depends on multiple unresolved
wrappers is not considered healthy. The recovery commands above are part of the
normal installation contract, not an optional afterthought.
