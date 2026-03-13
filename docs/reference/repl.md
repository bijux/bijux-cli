# REPL

`bijux repl` provides an interactive shell on top of the same parser, routing graph, execution kernel, and output envelopes used by non-interactive CLI execution.

## Guarantees

- Command routing parity with CLI.
- Stable meta-command prefix `:`.
- History persistence with configurable cap.
- Deterministic completion for built-ins and plugin hooks.
- Interrupt and EOF-safe control flow.

## Runtime Controls

- `:help <command>`
- `:set trace on|off`
- `:set quiet on|off`
- `:set format json|yaml|text`
- `:plugin reload` (only when explicitly enabled by safety policy)
- `:exit`

## Command Surface

Built-in commands exposed in the REPL reference tests:

- `status`
- `doctor`
- `version`
- `:help <command>`
- `:set trace on|off`
- `:set quiet on|off`
- `:set format json|yaml|text`
- `:plugin reload`
- `:exit`

## Diagnostics

REPL sessions expose diagnostics dump and startup budget checks for latency and memory.

## Related Pages

- [Routing and surfaces architecture](../04-architecture/routing-and-surfaces.md)
- [Execution pipeline](../04-architecture/execution-pipeline.md)
- [Current Python behavior inventory](current-python/index.md)
