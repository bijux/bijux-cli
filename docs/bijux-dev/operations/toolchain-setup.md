---
title: Toolchain Setup
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Toolchain Setup

Local verification is trustworthy only when the command, toolchain, dependency
set, and artifact boundary match the repository contract. This page separates
source requirements from hosted automation so a green local run is not
misreported as CI parity.

## Toolchain Authorities

| Concern | Authority | Current requirement |
| --- | --- | --- |
| Rust compiler | `rust-toolchain.toml` | `1.86.0`, minimal profile |
| Rust components | `rust-toolchain.toml` | `clippy`, `rustfmt` |
| package MSRV | workspace `Cargo.toml` | `1.86` |
| Python | `crates/bijux-cli-python/pyproject.toml` | CPython 3.11 or newer |
| Python environment | `makes/_internal.mk` | `artifacts/python/.venv` |
| Python dependencies | `PYTHON_EDITABLE_SPEC` in `makes/_internal.mk` | editable package with test, lint, security, docs, and build extras |
| Rust artifacts | `.cargo/config.toml` and Rust gates | `artifacts/rust/` |
| documentation | `mkdocs.yml` and Python docs extras | MkDocs 1.x with pinned-compatible plugins |

`rustup` reads `rust-toolchain.toml` when commands run in the checkout. Verify
the result instead of relying on the shell’s default toolchain:

```bash
rustc --version
cargo --version
rustup component list --installed --toolchain 1.86.0
```

## System Prerequisites

Install these outside the repository before bootstrapping:

- Git;
- GNU Make;
- Rustup with the pinned toolchain available;
- CPython 3.11 or newer with `venv`;
- a C/C++ build toolchain and platform linker required by Rust and Maturin.

Network access is required when Rust or Python dependencies are not already
cached. Container, Kubernetes, SLURM, or platform-specific workflow tests have
additional environment requirements and are not part of baseline setup.

## Bootstrap

From repository root:

```bash
make bootstrap
make doctor-rs
cargo check --workspace --all-targets --locked
cargo run -q -p bijux-dev --bin bijux-dev-cli -- \
  quickcheck --format json --no-pretty
```

`make bootstrap` creates `artifacts/python/.venv`, upgrades its packaging
tools, and installs `crates/bijux-cli-python` with the repository’s development
extras. It may migrate or remove a legacy root `.venv`; the root environment is
not the supported location.

Bootstrap does not install Rust cargo subcommands. `make doctor-rs` verifies
Cargo, gate scripts, nextest selection inputs, and policy files. Each Rust gate
also refuses to run when its required cargo subcommand is missing.

## Rust Gate Tools

| Gate | Additional command |
| --- | --- |
| `make test-rs`, `make test-slow`, `make test-all` | `cargo-nextest` |
| `make audit` | `cargo-deny`, `cargo-audit` |
| `make coverage` | `cargo-nextest`, `cargo-llvm-cov` |

The GitHub helper targets pin the tools used by managed CI:

```bash
make gh-test-install-rust-tools
make gh-security-install-rust-tools
```

Those targets currently install `cargo-nextest 0.9.100`,
`cargo-deny 0.18.3`, and `cargo-audit 0.22.1`. Coverage requires
`cargo-llvm-cov`, but the repository does not currently define a pinned local
installer for it. Record the installed version when coverage evidence is
reviewed; do not claim exact tool parity where the repository has not governed
one.

## Documentation Environment

The Python development extras include MkDocs and all configured plugins.
`make docs-check` installs the documented requirements into the managed
environment, synchronizes governed docs inputs, performs a strict MkDocs build,
checks publication boundaries and navigation, and writes the site under
`artifacts/docs/`.

```bash
make docs-require
make docs-check
```

Use `make docs-require` to distinguish a missing tool or input from a content
failure. Do not install a second root `.venv` or write a `site/` directory at
repository root.

## Hosted Automation Alignment

The source checkout currently has more than one hosted toolchain declaration:

| Surface | Declared Rust |
| --- | --- |
| source toolchain and MSRV | `1.86.0` / `1.86` |
| repository governance workflow | `1.86.0` |
| release-validation workflow | `1.86.0` |
| docs deployment configuration | `1.86.0` |
| synchronized generic CI workflow | `1.86.0` |
| synchronized release environment | `1.86.0` |

These hosted declarations align with the source contract at Rust 1.86.0.
Alignment prevents CI and publication jobs from validating the repository with
a compiler below the workspace MSRV, but a local result still does not
establish hosted parity unless the operating system, installed tools, and
workflow environment also match.

`.github/release.env`, synchronized workflows, and shared checksums are managed
from `bijux-std`; do not edit them directly in this repository. The durable
invariant is to update the upstream repository manifest when the workspace MSRV
changes, merge that standards change, refresh this repository from the accepted
GitHub commit, and validate the shared checksum in the same change set.

Audit alignment directly when toolchain policy changes:

```bash
rg -n '1\.85|1\.86|RUST_TOOLCHAIN|rust_toolchain' \
  rust-toolchain.toml Cargo.toml .github
```

## Failure Diagnosis

| Symptom | Check first |
| --- | --- |
| wrong compiler or component | `rustc --version`, `rustup show active-toolchain` |
| Cargo output outside artifacts | `.cargo/config.toml`, `CARGO_TARGET_DIR` |
| Python import or MkDocs failure | `artifacts/python/.venv/bin/python`, install logs under `artifacts/python/install/` |
| missing nextest or security command | the gate’s explicit missing-tool error |
| local pass but hosted failure | exact workflow toolchain, OS, installed cargo tools, and environment |
| release toolchain below MSRV | managed `.github/release.env` and upstream `bijux-std` manifest |

Preserve command output under `artifacts/` and report the exact failing
boundary. Recreating environments without first recording the mismatch makes
toolchain failures harder to diagnose.

## Repository Anchors

- `rust-toolchain.toml`
- `Cargo.toml`
- `.cargo/config.toml`
- `makes/_internal.mk`
- `makes/rust.mk`
- `makes/bin/run_core_rust_gate.sh`
- `makes/docs.mk`
- `.github/release.env`
- `.github/workflows/ci.yml`
- `.github/workflows/repository-governance.yml`

Continue with [Repository Gates](repository-gates.md) after the environment is
known, and [CI and Automation](ci-and-automation.md) when the question concerns
hosted execution rather than local setup.
