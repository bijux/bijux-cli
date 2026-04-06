# Installation

Install bijux-dag and verify it works before creating your first graph.

## Platform and toolchain assumptions

This guide assumes:

- Linux or macOS,
- a shell environment (`bash` or `zsh`),
- one of: prebuilt binary access, Cargo install path, or local source build.

If your environment differs, verify command paths and permissions first.

## What gets installed

You install one CLI binary (`bijux-dag`) that exposes command families like `dag`, `run`, `inspect`, `replay`, and `diff`.

Runtime state is created when you execute runs (run records, artifact references, and diagnostics under repository/runtime state paths). Installation alone does not create run evidence.

## Install paths

Binary install (preferred when available):

```bash
install -m 0755 ./bijux-dag "$HOME/.local/bin/bijux-dag"
```

Cargo install:

```bash
cargo install bijux-dag
```

Local source build:

```bash
cargo build --release
./target/release/bijux-dag --help
```

## Exact post-install verification flow

Run this sequence and confirm all commands exit successfully:

```bash
bijux-dag --help
bijux-dag --version
bijux-dag run --help
```

Expected output pattern:

```text
- top-level help lists command families (dag/run/artifact/inspect/diff/replay/bundle)
- version command prints a version string
- run --help prints run command usage
```

## Fast failure diagnosis

- `command not found`: binary directory is not on `PATH`.
- wrong version: stale binary earlier in `PATH`; check with `command -v bijux-dag`.
- Cargo install succeeded but CLI unavailable: ensure `$HOME/.cargo/bin` is in `PATH`.

## Next reading

- Create and run your first graph: [First Dag](../02-getting-started/02-first-dag.md)
- Immediate debugging flow when setup fails: [Basic Troubleshooting](../02-getting-started/05-basic-troubleshooting.md)
