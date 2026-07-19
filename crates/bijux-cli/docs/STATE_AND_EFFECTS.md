# State And Effects

`bijux-cli` reads and mutates local runtime state, but stateful behavior is not
allowed to leak into parsing or routing. This document defines where effects
belong and what callers may rely on.

## Effect Classes

| Effect | Owning area | Required behavior |
| --- | --- | --- |
| environment reads | infrastructure/install adapters | capture precedence and provenance |
| configuration files | config feature through filesystem adapters | validate before mutation |
| history and memory | owning features through state stores | preserve ordering and explicit absence |
| plugin manifests | plugin feature and registry adapters | validate before dispatch |
| subprocesses | plugin or mounted-app execution adapters | preserve executable and status context |
| terminal streams | interface/bootstrap | keep stdout and stderr contracts distinct |
| telemetry | kernel/shared telemetry | bound payloads and avoid semantic mutation |
| locks and migrations | install APIs | fail explicitly on contention or invalid state |

Pure code may build plans, validate schemas, normalize paths, and classify
errors. It must not perform hidden IO while doing so.

## Configuration Resolution

Configuration is resolved once into owned contract types. Source provenance is
retained so diagnostics can explain whether a value came from defaults, files,
environment, project state, a profile, or explicit invocation flags.

Writers must:

- validate the candidate document before replacing durable state;
- avoid partially written configuration;
- preserve unrelated valid keys;
- report conflicts rather than selecting a winner silently;
- return the path and mutation result needed for diagnosis.

## Runtime State Paths

State-path discovery is exposed through `api::install` because the Python
distribution and diagnostics need the same authority. Home-directory,
environment, file-config, and explicit override handling must remain
consistent across native and Python consumers.

Callers should not concatenate their own config, history, plugin, or lock paths.
Doing so bypasses migration, normalization, and compatibility behavior.

## External Execution

Plugins and mounted applications run only after the runtime has:

1. selected a canonical route;
2. validated the descriptor and compatibility window;
3. resolved the executable or interpreter;
4. established stream and timeout policy.

External stdout is accepted as structured command output only when it satisfies
the expected envelope contract. Incidental logs belong on stderr. A malformed
payload cannot be converted into an empty successful result.

## Failure And Recovery

State and effect failures include actionable context without exposing secrets.
Messages should identify the operation and relevant safe path, executable, or
status class. Recovery must be explicit:

- a missing optional registry may yield an empty governed registry;
- malformed durable state is an error;
- lock contention is an error, not permission to write without locking;
- a failed migration leaves the previous valid state intact;
- subprocess timeout and non-zero exit remain separate outcomes.

## Testing Effects

Use temporary directories and explicit environment maps in tests. Avoid tests
that depend on the developer's home directory, installed plugins, shell
configuration, locale, or unordered filesystem traversal.

Relevant evidence lives in:

- install and config focused suites under `tests/integration.rs`;
- plugin lifecycle and mounted-app suites under `tests/integration.rs`;
- path and compatibility tests in `bijux-cli-python`;
- kernel tests for cancellation, timeout, stream, and telemetry behavior.

Run the narrow suite that owns the changed effect. Architecture checks must
also pass when introducing a new adapter or process boundary.
