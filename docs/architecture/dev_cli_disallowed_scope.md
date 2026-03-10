# Dev CLI Disallowed Scope

`bijux-dev-cli` must not own runtime command law.

Disallowed ownership:
- Core routing/parser command law for non-`dev cli` surfaces.
- Runtime state mutation implementations.
- Plugin lifecycle/runtime execution law.
- Output-law/exit-law policy for runtime commands.
- Direct replacement of the canonical `bijux` binary surface.

If a behavior changes user-facing runtime law, it belongs in runtime crates and is consumed by `bijux-dev-cli` through structured query interfaces.
