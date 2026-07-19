# Runtime Selection And Failure Semantics

`bijux-cli-python` exposes one Python API over multiple transports. The native
extension and subprocess fallback must preserve the same `bijux-cli` command
semantics. The DAG SDK is separate: it is always a process client for an
independently installed `bijux-dag` executable.

## Selection Model

```mermaid
flowchart TB
    call["Python API call"]
    kind{"Requested surface"}
    native{"Native extension loaded?"}
    bridge["PyO3 bridge to bijux-cli"]
    binary["Resolve bijux executable"]
    process["Sanitized subprocess fallback"]
    dag["Resolve bijux-dag executable"]
    dag_process["bijux-dag --json subprocess"]
    outcome["Preserved result or classified failure"]

    call --> kind
    kind -->|root runtime| native
    native -->|yes| bridge --> outcome
    native -->|no| binary --> process --> outcome
    kind -->|DAG SDK| dag --> dag_process --> outcome
```

The fallback changes transport, not authority. It must not implement command
behavior, invent command names, or translate a failed runtime call into a
successful Python result.

## Native Import Policy

The package imports `bijux_cli_py._native` during facade initialization.
Missing-extension errors may select the process fallback. Other loader failures
are strict by default because they can indicate a broken or incompatible wheel.

| Condition | Default behavior |
| --- | --- |
| native module is absent | permit process fallback |
| strict mode is enabled | propagate the import failure |
| loader raises another `ImportError` | propagate unless it identifies the missing native module |
| loader raises `OSError` | propagate unless explicit loader fallback is enabled |

`BIJUX_PY_STRICT_IMPORT` controls strict behavior directly. Without an explicit
value, `BIJUX_ENV=dev`, `test`, or `ci` enables strict imports.
`BIJUX_PY_ALLOW_NATIVE_OSERROR_FALLBACK=1` is an explicit operational escape
hatch for loader failures; it must not become the default because it can hide
an invalid wheel.

Call `ensure_native_extension()` when a consumer specifically requires
in-process execution. General command calls may use either parity-preserving
transport.

## Root Runtime Resolution

When native execution is unavailable, `_runtime.resolve_runtime_binary`
selects `bijux` in this order:

1. `BIJUX_BIN`, which must identify an executable file;
2. recognized workspace target directories for repository development;
3. `bijux` from `PATH`, excluding the current Python console entrypoint to
   prevent recursive self-invocation.

No candidate is accepted merely because its path exists. It must be a regular
executable file after path expansion and resolution. Installed consumers
should use the packaged runtime or `BIJUX_BIN`; workspace discovery is a
development convenience, not a deployment contract.

The DAG client uses the same resolver with `BIJUX_DAG_BIN` and `bijux-dag`.
Installing the Python package does not satisfy that executable dependency.

## Process Boundary

Fallback execution captures stdout and stderr, preserves the process exit code,
and uses a bounded timeout. Before spawn, it removes Python injection and
dynamic-loader variables including `PYTHONHOME`, `PYTHONPATH`, `PYTHONSTARTUP`,
`LD_PRELOAD`, and `DYLD_INSERT_LIBRARIES`.

The root fallback timeout is configured by `BIJUX_PY_SUBPROCESS_TIMEOUT`; the
DAG client uses `BIJUX_DAG_PY_SUBPROCESS_TIMEOUT`. Missing, malformed, zero, or
negative values select the governed default rather than disabling the bound.

## Failure Classification

| Failure | Result |
| --- | --- |
| no compatible binary | `PlatformWheelUnavailable` with install or override guidance |
| binary candidate is missing or not executable | reject that explicit candidate |
| process timeout or spawn failure | exit `1`, preserved diagnostic, `InternalError` |
| process exit `2` | `UsageError` |
| process exit `3` | `ValidationError` |
| other non-zero exit | classify from governed stderr semantics, otherwise `InternalError` |
| malformed native outcome JSON | `InternalError` |
| native bridge lacks a required method | `NativeExtensionUnavailable` with reinstall guidance |
| malformed DAG JSON envelope | DAG transport contract failure |

Command-tree introspection is deliberately conservative without native
support. It asks the runtime for structured inspection and returns an explicit
warning with an empty namespace set when that query fails. It does not ship a
stale Python-owned copy of the command tree.

## Parity Invariants

- `--version` and `-V` normalize to the canonical `version` route.
- Native and process paths preserve exit code, stdout, stderr, and coarse error
  kind.
- Config and plugin helpers return the same field meanings across transports.
- Python exceptions classify runtime failures; they do not erase the original
  diagnostic.
- DAG helpers preserve the `bijux-dag --json` envelope and never call DAG
  runtime internals directly.
- A transport change cannot widen supported commands or compatibility claims.

## Verification

`tests/bridge_execution_parity.rs` compares native bridge execution with the
Rust authority. `tests/runtime_entrypoint_unity.rs` protects the single
runtime entrypoint. Python tests in `tests/python/test_runtime_resilience.py`
cover import policy, resolution precedence, recursion avoidance, environment
sanitization, timeout, and error classification.
`tests/python/test_dag_sdk_transport.py` protects the independent DAG process
boundary.
