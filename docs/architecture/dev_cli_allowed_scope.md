# Dev CLI Allowed Scope

`bijux-dev-cli` is the maintainer control-plane crate for `bijux dev cli ...`.

Allowed ownership:
- Maintainer workflow orchestration for `dev cli` commands.
- Maintainer report assembly and rendering.
- Machine-readable and text control-plane report envelopes.
- Maintainer command registry and metadata for `dev cli` namespace.
- Aggregation of runtime query outputs into maintainer diagnostics.

Required boundaries:
- Runtime command law stays in runtime crates.
- `bijux` remains the only canonical public binary.
- `bijux-dev-cli` must not become a second runtime command center.
