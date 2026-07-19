---
title: Execution Model
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Execution Model

The `bijux` process separates command interpretation from stream emission.
Core execution returns an `AppRunResult` containing an exit code, stdout, and
stderr. The binary entrypoint writes those streams and maps the integer result
to an operating-system exit code. This separation is what allows integration
tests and embedded callers to exercise the same routing behavior without
capturing a child process.

## Invocation Paths

| Entry | Responsibility | Boundary |
| --- | --- | --- |
| native binary | decode operating-system arguments, offer interactive mode, call the app runner, emit streams | invalid UTF-8 arguments fail before command parsing |
| `run_app` | interpret argv, resolve the route, execute it, render output, and return `AppRunResult` | does not itself write process stdout or stderr |
| interactive session | parse REPL input and preserve session exit state | REPL meta-commands are not ordinary root routes |
| delegated product or plugin process | execute another binary and capture its native result | delegated exit code and streams remain authoritative |

The binary and in-process paths must agree on command semantics. They differ
only at the process boundary: argv decoding, stream emission, and final
operating-system exit conversion.

## Dispatch Lifecycle

| Order | Decision | Owning code |
| --- | --- | --- |
| 1 | decode OS argv and reject non-UTF-8 input | `bootstrap/wiring.rs` |
| 2 | enter the REPL only when interactive invocation rules match | `bootstrap/repl.rs` |
| 3 | handle no-argument help and the `--version` alias | `interface/cli/dispatch.rs` |
| 4 | render explicit help or delegate help for a known Bijux tool | `interface/cli/dispatch/help.rs`, `dispatch/delegation.rs` |
| 5 | let Clap render recognized help or usage errors | `interface/cli/parser.rs` |
| 6 | parse global flags and normalize aliases into a command path | `routing/parser.rs` |
| 7 | execute install compatibility commands or resolve a registered route | `dispatch.rs`, `dispatch/route_exec.rs` |
| 8 | render a structured payload or preserve delegated process streams | `shared/output.rs`, `dispatch/route_exec.rs` |
| 9 | record bounded telemetry when enabled and return `AppRunResult` | `shared/telemetry.rs` |
| 10 | emit streams and normalize the process exit code | `bootstrap/run.rs` |

Help and delegation occur before ordinary route execution by design. A help
request must not initialize mutable state or run a command handler merely to
describe the surface.

## Route Resolution

The parser produces both the requested command path and a normalized path.
Alias rewriting happens before route lookup so help, execution, suggestions,
and telemetry can refer to one canonical route.

`RouteRegistry` begins with built-in namespaces. Plugin namespaces are loaded
only for plugin inspection commands or after an initial unknown-route result.
This avoids reading plugin state for unrelated built-in commands while still
allowing an installed plugin namespace or alias to resolve.

After resolution:

- a plugin target invokes the registered plugin runtime;
- config, history, memory, plugin-management, CLI, and root handlers are tried
  in owned order;
- built-in handlers return a structured `serde_json::Value`;
- plugin execution may return either a structured value or a native process
  result;
- an unresolved path becomes a usage-class error with bounded deterministic
  suggestions when a close route exists.

Route handlers own behavior. Dispatch owns ordering, formatting, stream
placement, and exit classification; it must not reimplement feature logic.

## Rendering And Streams

Output policy is resolved once from global flags and terminal context:

- an interactive stdout defaults to text; redirected stdout defaults to JSON;
- explicit JSON, JSON Lines, YAML, or text selection overrides that default;
- compact, color, `NO_COLOR`, log level, and quiet flags shape emission;
- successful built-in payloads render to stdout and receive a final newline;
- classified command failures render to stderr and leave stdout empty;
- delegated process results preserve the delegated stdout, stderr, and exit
  code;
- quiet mode suppresses successful built-in output without changing outcome.

`cli version` and completion scripts have intentional text renderers. They do
not pass through the generic structured renderer in text mode.

## Result And Telemetry Boundaries

Telemetry is observational and opt-in. It records bounded route, status,
stream-size, and exit information; it must not alter routing or expose
unbounded user input. A telemetry failure must not become command behavior.

The app runner can still perform command-owned filesystem or subprocess
effects. Returning streams in memory does not make command execution pure.
State mutation and plugin process boundaries are documented separately.

## Execution Invariants

- Help and usage paths do not execute the described command.
- The normalized route used for execution is the route reported in structured
  diagnostics.
- Built-in success does not write a success payload to stderr.
- A delegated process keeps its own nonzero outcome rather than being rendered
  as built-in success.
- Quiet mode changes emission, not command status or mutation.
- Unknown-route suggestions are deterministic, bounded, and never treated as
  successful dispatch.
- Binary and in-process execution preserve equivalent streams and exit
  outcomes for the same supported argv and state.

## Verification Anchors

- `crates/bijux-cli/src/bootstrap/run.rs`
- `crates/bijux-cli/src/bootstrap/wiring.rs`
- `crates/bijux-cli/src/interface/cli/dispatch.rs`
- `crates/bijux-cli/src/interface/cli/dispatch/route_exec.rs`
- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/tests/integration/cli/root/flag_normalization_laws.rs`
- `crates/bijux-cli/tests/integration/cli/root/root_command_coverage.rs`
- `crates/bijux-cli/tests/integration/cli/root/bin_core_integration.rs`

## Related Architecture

- [Error Model](error-model.md)
- [State and Persistence](state-and-persistence.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
