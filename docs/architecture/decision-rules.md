# Decision Rules (Behavior Constitution)

Purpose: define where CLI behavior is decided and where it must not be decided.

## What decides behavior
- `core/precedence.py` is the single source of truth for resolving flags, env, and defaults into an `ExecutionPolicy`.
- `core/exit_policy.py` is the single source of truth for exit codes, streams, and error shape.
- `cli/core/color.py` is the single source of truth for color/styling decisions.
- Command handlers are responsible only for building intent/payload and invoking emitters.

## What must not decide behavior
- Individual commands must not re-derive output policy (quiet/format/pretty/log policy).
- Individual commands must not hardcode raw flag strings or env var names.
- Helper utilities must not short-circuit output/error contracts outside `exit_policy`.
- DI/services must not invent new verbosity switches or logging rules.

## Enforcement
- Architecture tests forbid raw flag strings outside `cli/core/constants.py`.
- Property tests lock `LogLevel` ordering and parsing semantics.
- CI gates ensure policy changes only flow through the core policy modules.
