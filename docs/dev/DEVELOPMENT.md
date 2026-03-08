# Development Environment and Tooling

## Rust toolchain

- Supported Rust version: `1.81`.
- Workspace metadata uses `rust-version` and package metadata in `Cargo.toml`.
- Pin toolchain in `rust-toolchain.toml` so contributors build against a predictable baseline.

## Build artifact layout

- Build artifacts are written to `artifacts/target` via `.cargo/config.toml`.
- The path is intentionally repository-local and excluded from source history through `artifacts/.gitignore`.
- Never commit files under `artifacts/`.

## Cache and target directory policy

- Local cache and compile outputs are isolated under `artifacts/`.
- Temporary cache directories are ephemeral and may be removed with:
  - `cargo run -p bijux-dev-dag -- artifacts-clean`
  - Or manually:
    - `rm -rf artifacts/target`
- For CI, the same override is respected by all dev workflow commands.
- To force a different target directory, set `CARGO_TARGET_DIR` explicitly.

## Profiles and execution modes

- Use default `dev` profile for local development and iteration.
- Use CI profile behavior through the dedicated CI entry points in `bijux-dev-dag` rather than manual flag combinations.

## Required developer tools

- Install required tools with:
  - `rustup component add rustfmt clippy`
  - `cargo install cargo-audit`
  - `cargo install cargo-public-api`
  - `cargo install cargo-nextest` (for local parallelized test workflows)
- The repository keeps a `cargo nextest` baseline in `configs/nextest/nextest.toml`.

## Tooling policy

- `cargo` commands used by repository policy:
  - `cargo fmt` and `cargo clippy` for style and lint enforcement.
  - `cargo audit` for dependency vulnerability checks.
  - `cargo public-api` for API surface diffs and `bijux-dev-dag` guarding.
  - `cargo nextest run` for local/CI test orchestration when available.
