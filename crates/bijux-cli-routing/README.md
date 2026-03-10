# bijux-cli-routing

Command graph, namespace matching, and route selection.

## Boundary
- Depends on `bijux-cli` and `bijux-cli-routing`.
- Must not depend on plugin loading or output rendering internals.

## Parser Choice
- Uses `clap` as the canonical parser for Rust command surfaces.
