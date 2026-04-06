# Workspace Ownership Map

`bijux-core` uses a single Rust workspace rooted at repository `Cargo.toml`.

## Product Layers

- `crates/bijux-cli`: primary runtime and command surface authority.
- `crates/bijux-cli-python`: Python packaging and bridge for the runtime.
- `crates/bijux-dev-cli`: maintainer control plane for repository quality gates.
- `bijux-dag/crates/*`: DAG engine, app, cli, and testkit crates integrated as peer workspace members.

## Ownership Rules

- Root workspace manifest is the only source for workspace members and shared dependencies.
- DAG crates are additive capabilities and do not redefine workspace authority.
- Cross-crate dependencies use local `path` wiring and workspace-inherited metadata.
- Build orchestration is exposed from root `Makefile` and `makes/*`.

## Path Contract

- Runtime crates stay under `crates/`.
- DAG domain crates stay under `bijux-dag/crates/` to keep DAG docs, evidence, and configs colocated.
- Repository-level docs live under `docs/` with links into DAG-specific docs under `bijux-dag/docs/`.

## Operational Contract

Use root commands for all workspace actions:

```bash
cargo check --workspace
cargo test --workspace
make dag-test
make dag-contracts
```

This contract keeps contributor workflows deterministic and avoids split workspace drift.
