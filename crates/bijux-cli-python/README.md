# bijux-cli Python Package

<!-- bijux-core-badges:generated:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)](https://pypi.org/project/bijux-cli/)
[![PyPI](https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi)](https://pypi.org/project/bijux-cli/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli-python docs](https://img.shields.io/badge/docs-bijux--cli--python-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli-python/)
<!-- bijux-core-badges:generated:end -->

`bijux-cli-python` is the Python distribution for installing and launching the Bijux
`bijux` command through PyPI. It provides the Python launcher, native bridge,
fallback facade, mounted-app SDK, and a process client for an independently
installed `bijux-dag` executable.

Installing this wheel does **not** install the `bijux-dag` binary. The DAG
helpers are clients of that binary; they are not an in-process workflow engine
and do not define independent DAG semantics.

## Install And Verify

```bash
python -m pip install bijux-cli
bijux --help
bijux doctor
python -m bijux_cli_py --help
```

The package is on the `v0.4.0` release line and requires Python 3.11 or newer.
The `bijux` console entrypoint and `python -m bijux_cli_py` resolve the same
runtime contract.

## Package Boundary

| Surface | This package owns | Upstream authority |
| --- | --- | --- |
| Python installation | wheel and source-distribution metadata, `bijux` entrypoint, platform compatibility diagnostics | `pyproject.toml` and packaging tests |
| native bridge | Python/Rust conversion and invocation boundary | `bijux-cli` runtime semantics |
| fallback facade | compatible Python access when the native bridge is unavailable | the same public `bijux` command contract |
| mounted apps | descriptor construction and root-compatible result envelopes | `bijux` discovery and routing policy |
| DAG helpers | argument construction, executable resolution, timeout, and JSON-envelope decoding | installed `bijux-dag` command |

This package does not own maintainer commands, repository governance, a
Python-only command schema, or a Python implementation of the DAG runtime.

## Runtime Resolution

The Python launcher and DAG client resolve different executables:

| Client | Required executable | Override |
| --- | --- | --- |
| `bijux` launcher/facade | packaged or discoverable `bijux` runtime | runtime resolution owned by `bijux_cli_py._runtime` |
| `bijux_cli_py.dag_sdk` | `bijux-dag` on `PATH` | `BIJUX_DAG_BIN` |

Repository checkouts may resolve workspace binaries for development. Installed
applications should treat `PATH` or the explicit override as the deployment
contract, not depend on checkout discovery.

`BIJUX_DAG_PY_SUBPROCESS_TIMEOUT` controls the DAG process timeout. The selected
binary defines available commands and release behavior.

## DAG Workflow Client

Install the DAG executable separately before using `dag_sdk`:

```bash
cargo install bijux-dag-cli
bijux-dag version
```

| Requirement | Behavior |
| --- | --- |
| executable | `bijux-dag` must be on `PATH`, or `BIJUX_DAG_BIN` must name the executable |
| transport | every operation invokes `bijux-dag --json` as a subprocess |
| output | helpers preserve the CLI JSON envelope rather than inventing a Python-only schema |
| temporary input | dictionary graph inputs are written to a temporary JSON file and removed after the command |
| failures | binary resolution, timeout, non-zero command status, and malformed JSON remain distinguishable |

```python
from pathlib import Path

from bijux_cli_py import validate_dag_graph

result = validate_dag_graph(Path("workflow.dag.json"))
```

Use the [DAG command surface](../../docs/bijux-dag/interfaces/cli-surface.md)
for supported behavior and the
[release boundary](../../docs/bijux-dag/foundation/release-boundary.md) before
depending on a non-stable route.

## Mounted Python Apps

`bijux_cli_py.app_sdk` builds mount descriptors and root-compatible result
envelopes for Python applications. Discovery policy and route execution remain
owned by the Rust `bijux` runtime.

```python
from bijux_cli_py.app_sdk import build_python_mount_manifest

manifest = build_python_mount_manifest(
    namespace="sample",
    display_name="Sample App",
    module="sample_app.cli",
    function="main",
    summary="Sample mounted Python app",
)
```

The [mounted app guide](./docs/MOUNTED_APPS.md) owns callable shape,
compatibility checks, manifest placement, stream discipline, and packaging.

## Source Map

| Path | Responsibility |
| --- | --- |
| `python/bijux_cli_py/cli.py` | Python console entrypoint |
| `python/bijux_cli_py/_facade.py` | public facade and native/fallback selection |
| `python/bijux_cli_py/_runtime.py` | executable discovery, subprocess execution, timeout, and error classification |
| `python/bijux_cli_py/app_sdk.py` | mounted-app descriptors and result envelopes |
| `python/bijux_cli_py/dag_sdk.py` | typed `bijux-dag --json` process client |
| `src/lib.rs` | PyO3 native bridge |
| `tests/python/` | Python packaging, parity, app SDK, and DAG transport contracts |

The runtime implementations live outside this crate:

- `crates/bijux-cli` owns `bijux` command semantics.
- `crates/bijux-dag-runtime` owns DAG execution behavior.
- `crates/bijux-dag-cli` owns the thin `bijux-dag` executable boundary.

## Failure Decisions

- Run `bijux doctor` when the installed `bijux` runtime cannot be resolved.
- Use `dag_post_install_diagnostics()` before offering DAG features from a
  Python application.
- Treat an unavailable `bijux-dag` executable as a deployment error.
- Treat invalid JSON from `bijux-dag` as a runtime contract failure, not a
  successful empty result.
- Preserve the original structured command failure for diagnosis.

## References

- Runtime crate: [`crates/bijux-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli)
- DAG runtime crate: [`crates/bijux-dag-runtime`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-runtime)
- DAG executable crate: [`crates/bijux-dag-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli)
- Python bridge crate: [`crates/bijux-cli-python`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli-python)
- Mounted app guide: [`crates/bijux-cli-python/docs/MOUNTED_APPS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli-python/docs/MOUNTED_APPS.md)
- Package changelog: [`crates/bijux-cli-python/CHANGELOG.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli-python/CHANGELOG.md)
- Repository handbook: [CLI handbook](https://bijux.io/bijux-core/bijux-cli/)

## License

Apache-2.0 ([repository LICENSE](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
