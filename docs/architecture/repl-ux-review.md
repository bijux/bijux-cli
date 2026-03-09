# REPL UX Review

Date: 2026-03-09

## What works well in current baseline

1. Session controls are explicit and discoverable (`:help`, `:set`, `:exit`).
2. Output format switching is immediate and deterministic.
3. Quiet and trace toggles are clear and predictable.
4. Error recovery allows continuing the same session after usage/syntax failures.
5. Completion covers root commands, grouped namespaces, plugin hooks, and partial tokens.

## UX constraints accepted for baseline

1. Prompt-toolkit-specific visual UX from Python is not replicated.
2. Advanced shell-like conveniences (semicolon chaining, richer piping helpers) are deferred.
3. Completion suggestions are prefix-based and contract-stable, not context-ranked.

## Safe follow-up improvements (post-baseline)

1. Context-aware completion ranking without changing stable command contracts.
2. Optional richer REPL help pages for grouped commands and plugin namespaces.
3. Optional startup diagnostics summary panel when plugin registry warnings are present.
