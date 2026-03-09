# Hermetic Builds

Hermetic builds are produced from pinned toolchains and repository-local sources, without implicit host state.

## Rules

- Pin Rust toolchain with `rust-toolchain.toml`.
- Build from a clean checkout.
- Avoid downloading mutable dependencies during build steps when an internal mirror or lockfile is available.
- Record artifact checksums for every release payload.

## Minimal Workflow

```bash
cargo build --locked --workspace
cargo test --locked --workspace
```

