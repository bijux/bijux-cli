# bijux-cli-routing

Command graph, namespace matching, and route selection.

## Boundary
- Depends on `bijux-cli-core` and `bijux-cli-contracts`.
- Must not depend on plugin loading or output rendering internals.

## Parser Choice
- Uses `clap` as the canonical parser for Rust command surfaces.
