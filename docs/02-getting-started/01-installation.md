# Installation

## Purpose
Provide reliable installation paths for bijux-dag and a quick verification flow.

## Context
This is the entrypoint for first-time setup before creating or running any DAG.

## Explanation
System requirements:
- OS: Linux or macOS.
- Shell: POSIX-compatible shell (`bash` or `zsh`) for examples in this guide.
- Tooling for source install: Rust toolchain + Cargo.
- Optional: Git, when installing from repository source.

Supported installation paths for local usage:
- prebuilt binary on `PATH`
- Cargo install from source package
- local repository build for development workflows

Recommended order:
1. Use prebuilt binary when available.
2. Use `cargo install` when binary distribution is not available.
3. Use local build when contributing or testing repository changes.

### Binary installation
If your release process provides a binary artifact, place it on `PATH` and verify:
- command resolves
- version command executes

Linux/macOS placement guidance:
- install into a directory already on `PATH` (for example `/usr/local/bin` or `$HOME/.local/bin`).
- ensure executable permission is set on the binary.

### Cargo installation
Use Cargo when you need a local install from Rust packaging.

Linux/macOS Cargo path note:
- Cargo-installed binaries are typically placed under `$HOME/.cargo/bin`.
- ensure that directory is available on `PATH`.

### Local build
Use local build when working inside the repository or validating unmerged changes.

Linux/macOS build note:
- local build is useful when testing branch changes before installation.
- invoke binary from `target/release` to validate the exact built artifact.

### Post-install verification
Always verify before continuing:
- CLI resolves (`--help` works)
- runtime command surface loads
- no missing dynamic dependency errors
- version command succeeds and reports expected binary

## Examples
```bash
# Verify CLI is resolvable
bijux-dag --help
bijux-dag --version

# Binary install example (path may vary by release process)
install -m 0755 ./bijux-dag "$HOME/.local/bin/bijux-dag"
bijux-dag --version

# Cargo install path
cargo install bijux-dag
bijux-dag --version

# Local repository build path
cargo build --release
./target/release/bijux-dag --help
./target/release/bijux-dag --version
```

```text
Expected successful verification:
- help text is printed
- version text is printed
- exit status is zero
```

## Guarantees
- Installation paths in this document are concrete and reproducible.
- Verification commands are minimal and safe to run repeatedly.

## Limitations
- This document does not define package manager integration beyond Cargo.
- Environment-specific toolchain issues are covered in troubleshooting.

## Related
- `docs/02-getting-started/02-first-dag.md`
- `docs/02-getting-started/05-basic-troubleshooting.md`
- `docs/07-operations/01-ci-integration.md`
- `docs/07-operations/02-reproducible-builds.md`
