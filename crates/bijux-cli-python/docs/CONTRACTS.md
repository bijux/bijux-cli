# `bijux-cli-python` Contracts

`bijux-cli-python` owns Python distribution and interop for the `bijux`
runtime. It does not own a second implementation of command semantics. Its
native bridge, fallback facade, launcher, and mounted-application SDK must
remain compatible with `bijux-cli`.

The package is private in the Cargo publication graph because Cargo builds the
native extension as part of Python distribution. The resulting Python package
is a public installation surface.

## Owned Surface

The package owns:

- wheel and source-distribution metadata;
- Python 3.11-or-newer compatibility;
- the `bijux` console launcher and `python -m bijux_cli_py`;
- PyO3 conversions and native-extension loading;
- fallback selection when the extension is unavailable;
- mounted Python application descriptors and result envelopes;
- the subprocess client for an independently installed `bijux-dag`.

It does not own `bijux` routing, DAG graph semantics, DAG execution, or
maintainer commands.

## Runtime Authorities

| Python surface | Semantic authority |
| --- | --- |
| launcher and facade | `bijux-cli` |
| native bridge conversions | exported Rust types from `bijux-cli` |
| mounted application SDK | `bijux-cli` mount discovery and routing |
| DAG process client | `bijux-dag-cli` JSON command surface |
| package metadata and interpreter support | `crates/bijux-cli-python/pyproject.toml` |

The fallback facade is a compatibility path, not an independent product
implementation. If it cannot preserve a supported operation, it must report
that limit rather than approximate native behavior.

## Process Boundary

The DAG helpers execute `bijux-dag --json`. They may resolve an executable,
construct arguments, materialize temporary graph input, enforce a timeout, and
decode the returned envelope. They must not:

- reinterpret a failed command as a successful Python value;
- invent a Python-only graph or response schema;
- keep temporary graph files after a completed call;
- search repository build directories in an installed deployment;
- hide which executable was selected when diagnosis is requested.

`BIJUX_DAG_BIN` is the explicit executable override.
`BIJUX_DAG_PY_SUBPROCESS_TIMEOUT` controls the client timeout. Neither changes
the selected binary's supported command surface.

## Invariants

- Native and fallback entrypoints preserve public command meaning.
- Rust-to-Python conversions preserve envelope status and diagnostic fields.
- Installed-package behavior does not depend on a source checkout.
- A missing executable, timeout, nonzero status, and malformed JSON remain
  distinct failure classes.
- Mounted application output remains compatible with the root runtime.
- Python packaging cannot imply that `bijux-dag` is bundled.

## Failure Contract

Extension import failure may select the documented fallback when compatibility
can be preserved. ABI incompatibility, executable resolution failure, timeout,
invalid output, or unsupported behavior must remain explicit. Diagnostics may
redact secrets, but cannot erase the causal executable, operation, or failure
class.

## Verification

| Claim | Required evidence |
| --- | --- |
| native bridge conversion | `crates/bijux-cli-python/tests/bridge_bindings.rs` |
| package and entrypoint ownership | `crates/bijux-cli-python/tests/python_packaging_ownership.rs` and `runtime_entrypoint_unity.rs` |
| Python package metadata | `crates/bijux-cli-python/tests/python/test_packaging_contracts.py` |
| native/fallback parity | `crates/bijux-cli-python/tests/python/test_runtime_parity.py` |
| DAG transport and workflow delegation | `test_dag_sdk_transport.py` and `test_dag_sdk_workflows.py` |

Run Rust bridge checks and the focused Python tests appropriate to the changed
surface. A full Python claim also requires the repository Python test lane; a
Rust-only run does not prove wheel, interpreter, or subprocess behavior.

Changes to launcher behavior, bridge conversions, fallback selection, DAG
delegation, or packaging metadata must update this page, the package README,
and parity evidence together.
