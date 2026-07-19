# Process And Exit Behavior

The executable preserves parser and application outcomes at the operating
system boundary. It must not reinterpret domain results.

## Argument Parsing

The wrapper clones the app-owned command to support completion generation,
then calls Clap once for the actual process arguments. Parser diagnostics,
usage text, and parser exit behavior are Clap/app contract surfaces.

No manual pre-parser should inspect route names or rewrite arguments.

## Dispatch Outcomes

`dag_run` returns:

- `Ok(code)` for a completed application dispatch;
- `Err(code)` for an application refusal or failure represented as status.

The wrapper returns either code unchanged. It does not convert nonzero status
to success, retry a command, or emit an additional error envelope.

## Stream Ownership

- Parser diagnostics use parser-selected streams.
- Application rendering uses app-selected streams.
- Completion scripts are written to stdout.
- Unexpected panic containment writes one internal error to stderr.

The wrapper must not combine stdout and stderr. JSON output must remain exactly
the app's parseable document.

## Exit Classes

The process preserves stable broad behavior:

| Outcome | Status behavior |
| --- | --- |
| successful command or completion | zero |
| argument or unsupported completion shell | usage-style nonzero |
| application refusal or domain failure | app-selected nonzero |
| unexpected panic | internal nonzero |

Exact domain classification belongs to the app. The wrapper cannot infer it
from output text.

## Signals And Cleanup

Long-running cleanup and cancellation are runtime responsibilities because
they require knowledge of active attempts and backends. The wrapper must not
install competing signal handlers that bypass runtime state transitions.

## Testing

Process tests should invoke the built `bijux-dag` binary with isolated paths
and assert status, stdout, and stderr. They should not depend on user
configuration, home state, installed adapters, or mutable shared run roots.

Use app-level tests for command semantics and wrapper tests only for behavior
observable at startup, parsing, completion, panic containment, or exit.
