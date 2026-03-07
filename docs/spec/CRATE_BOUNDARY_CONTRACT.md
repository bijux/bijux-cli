# Crate Boundary Contract

## Forbidden edges
- `bijux-dag-runtime -> bijux-dag-app`
- `bijux-dag-runtime -> bijux-dag-cli`
- `bijux-dag-core -> bijux-dag-runtime`
- `bijux-dag-core -> bijux-dag-app`

## Thin CLI rule
`bijux-dag-cli` is dispatch-only and must not implement execution, scheduling, or artifact semantics.

## App orchestration rule
`bijux-dag-app` may orchestrate runtime calls and output shaping, but must not contain scheduler internals.

## Runtime policy
Runtime owns execution semantics and consumes core/artifact services through typed interfaces.
