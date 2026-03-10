# bijux-cli-core

Core runtime types and execution primitives.

## Boundary
- Owns execution orchestration primitives.
- Depends only on `bijux-cli-routing`.
- Must not depend on transport-specific crates (`output`, `repl`, `python`).
