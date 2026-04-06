# bijux-core

`bijux-core` is the unified repository for the Bijux command runtime and DAG execution system.

## What lives here

- `crates/bijux-cli`: primary runtime crate and `bijux` binary.
- `crates/bijux-cli-python`: Python bridge and packaging surface.
- `crates/bijux-dev-cli`: maintainer control-plane.
- `bijux-dag/crates/*`: DAG core, runtime, app, CLI, testkit, and maintainer crates.

## Workspace model

This repository has one Rust workspace rooted at `Cargo.toml`.

All build, test, and release validation commands are expected to run from repository root.

## Quick start

```bash
cargo check --workspace
cargo test --workspace
cargo run -p bijux-cli --bin bijux -- --help
cargo run -p bijux-dag-cli -- dag --help
```

## Make targets

```bash
make help
make dag-help
make dag-test
make dag-contracts
```

## Documentation

- Runtime and control plane docs: `docs/`
- DAG domain docs: `bijux-dag/docs/`
- Workspace ownership map: `docs/04-architecture/workspace-ownership-map.md`

## Design intent

- Keep CLI runtime ownership explicit.
- Keep DAG capabilities integrated as peer workspace crates.
- Keep one source of truth for workspace settings, dependencies, and automation.

## License

Apache-2.0 (`LICENSE`).
