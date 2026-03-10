# Binary Ownership And Install Policy

## Cargo Naming Decisions

- Canonical crate identity is `bijux-cli`.
- Compatibility cargo package name `bijux` is supported as an alias channel.
- Both cargo install channels must resolve to the same executable: `bijux`.

## Python Naming Decisions

- Canonical Python package is `bijux-cli`.
- Compatibility/meta package name `bijux` may be published as an alias channel.
- Both pip install channels must resolve to the same executable: `bijux`.

## Executable Ownership Rule

`bijux-cli` is the sole owner of the public `bijux` command contract. Compatibility package names may exist, but they must delegate to the same command engine and must not publish divergent secondary executables.

## Install Strategy Matrix

- `cargo install bijux-cli` -> installs `bijux`
- `cargo install bijux` -> installs `bijux` (compatibility alias)
- `pip install bijux-cli` -> installs `bijux`
- `pip install bijux` -> installs `bijux` (compatibility alias)

## Tool Project Naming Contract

- Runtime tool binaries follow `bijux-<tool>`.
- Control-plane tool binaries follow `bijux-dev-<tool>`.
- Umbrella command routing follows:
  - `bijux <tool> ...` -> `bijux-<tool>`
  - `bijux dev <tool> ...` -> `bijux-dev-<tool>`
- The canonical tool registry is `KNOWN_BIJUX_TOOLS` and must match `docs/constitution/official_product_namespace_registry.json`.
