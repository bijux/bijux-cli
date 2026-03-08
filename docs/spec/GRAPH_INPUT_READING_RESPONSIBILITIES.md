# Graph Input Reading Responsibilities

## Scope
Defines app-layer ownership for loading graph input before command execution.

## Rules
1. Filesystem input reading is owned by `crates/bijux-dag-app/src/read/fs_input.rs`.
2. Graph parse and spec-compat normalization are owned by `crates/bijux-dag-app/src/read/read_graph.rs`.
3. Command entry routing in `crates/bijux-dag-app/src/lib.rs` must delegate to these readers and must not duplicate graph-load parsing logic.
4. Graph read failures must exit before runtime execution side effects.
