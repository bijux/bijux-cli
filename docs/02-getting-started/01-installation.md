# Installation

## Purpose
Provide reliable installation paths for bijux-dag and a quick verification flow.

## Context
This is the entrypoint for first-time setup before creating or running any DAG.

## Explanation
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

### Cargo installation
Use Cargo when you need a local install from Rust packaging.

### Local build
Use local build when working inside the repository or validating unmerged changes.

### Post-install verification
Always verify before continuing:
- CLI resolves (`--help` works)
- runtime command surface loads
- no missing dynamic dependency errors

## Examples
```bash
# Verify CLI is resolvable
bijux-dag --help

# Cargo install path
cargo install bijux-dag

# Local repository build path
cargo build --release
./target/release/bijux-dag --help
```

```text
Expected successful verification:
- help text is printed
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
