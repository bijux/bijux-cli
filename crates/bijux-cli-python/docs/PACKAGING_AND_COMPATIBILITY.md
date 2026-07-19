# Packaging And Compatibility

The Python release must install one coherent `bijux` command across supported
CPython environments. Packaging metadata, the native extension, Python
fallback code, and runtime compatibility behavior are one release unit.

## Build Contract

Maturin builds the distribution from:

- `pyproject.toml` for Python metadata, entrypoints, dependencies, and package
  policy;
- `Cargo.toml` for the `cdylib` and `python-extension` feature;
- `python/bijux_cli_py/` for pure-Python modules;
- `src/` for the Rust bridge.

The extension module name is `bijux_cli_py._native`. The build enables
PyO3's `abi3-py311` contract, matching the declared Python 3.11 minimum.
Changing either side requires wheel and interpreter-matrix verification.

## Entrypoint Ownership

The distribution declares exactly one console entrypoint:

```text
bijux = bijux_cli_py.cli:main
```

`python -m bijux_cli_py` reaches the same launcher authority. The package must
not ship a second parser or command registry. Native and fallback paths both
delegate to the canonical runtime behavior.

## Compatibility Paths

The Rust compatibility layer re-exports path behavior from `bijux-cli`.
Resolution considers:

1. explicit `PathOverrides`;
2. recognized environment variables;
3. compatibility configuration;
4. default paths derived from the selected home directory.

Paths are normalized against the home directory where required. Unknown
configuration keys are rejected. State locks, file initialization, and
migrations remain part of the canonical install API rather than Python-only
filesystem logic.

## Optional Runtime Dependencies

The root command and DAG helpers have different deployment contracts:

- the package installs the `bijux` Python launcher and its bridge/fallback;
- it does not install `bijux-dag`;
- `dag_sdk` requires `bijux-dag` on `PATH` or in `BIJUX_DAG_BIN`;
- mounted Python apps require an interpreter discoverable under root-runtime
  policy.

Diagnostics must name the missing component instead of reporting a generic
package failure.

## Release Compatibility

A releasable build verifies:

- metadata version and runtime version agreement;
- supported Python classifiers and `requires-python`;
- wheel contents and importability;
- console and module entrypoint unity;
- native/fallback command parity;
- stable release behavior without checkout-only paths;
- mounted-app descriptor compatibility;
- DAG transport behavior against an independently resolved executable.

The package is private as a Cargo crate but public as a Python distribution.
Cargo publishability therefore does not describe the Python release status.

## Failure Policy

Native extension import failure may use the governed fallback when the failure
class is allowed. Command failures after successful native loading are not
extension-loading failures and must not trigger a second execution path.

Installed behavior must not depend on:

- the repository checkout;
- a prebuilt extension left in the source tree;
- developer-specific `PATH` ordering;
- writable package directories;
- unbounded subprocess waits.

## Verification

```bash
cargo test --locked -p bijux-cli-python \
  --test python_packaging_ownership \
  --test runtime_entrypoint_unity

python -m pytest \
  crates/bijux-cli-python/tests/python/test_packaging_contracts.py \
  crates/bijux-cli-python/tests/python/test_stable_release_compatibility.py
```

Run native/fallback parity and resilience tests before changing extension
loading, executable discovery, timeout handling, or exception mapping.
