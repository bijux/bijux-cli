# bijux-cli-core

Core runtime types and execution primitives.

## Boundary
- Owns execution orchestration primitives.
- Depends only on `bijux-cli-contracts`.
- Must not depend on transport-specific crates (`output`, `repl`, `python`).
