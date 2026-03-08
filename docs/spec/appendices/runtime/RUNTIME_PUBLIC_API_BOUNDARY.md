# Runtime Public API Boundary

## Sacred public runtime surfaces
- execution engine orchestration entrypoints
- scheduler contract surfaces
- typed run and node result models
- policy and selector inputs
- trace and invariant outputs

## Public API restrictions
- speculative modules must not be re-exported as first-class public runtime APIs.
- roadmap or product strategy models remain internal support types.
- control-plane semantics remain outside runtime core ownership.

## Facade modules
- `runtime`: core runtime orchestration facade
- `adapters`: adapter-related runtime facade
- `execution`: execution-path facade (plan, backend, executor)
