# bijux-cli Python Package

<!-- bijux-core-badges:generated:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)](https://pypi.org/project/bijux-cli/)
[![PyPI](https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi)](https://pypi.org/project/bijux-cli/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli-python docs](https://img.shields.io/badge/docs-bijux--cli--python-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli-python/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-cli-python` is the Python distribution for installing and launching the
`bijux` command runtime, and for delegating DAG workflow calls to
`bijux-dag`.

It is the PyPI boundary for the public `bijux` product. The Python package
does not redefine runtime behavior; it packages, launches, and validates the
same command contracts owned by the Rust `bijux-cli` and `bijux-dag-cli`
crates.

## Release Status

- public PyPI distribution on the `v0.4.0` release line
- companion surface to the Rust `bijux-cli` crate
- not a separate command product from `bijux`

## What This Package Owns

- the `bijux` console entrypoint for Python installs
- the optional native Rust bridge module (`bijux_cli_py._native`)
- a Python fallback facade for compatibility and portability checks
- `bijux_cli_py.app_sdk` for mounted Python apps
- `bijux_cli_py.dag_sdk` for Python callers that need to load graphs, validate
  them, produce plans, run workflows, inspect run state, and query artifact
  registries through `bijux-dag`

## What It Does Not Own

- independent runtime semantics
- independent DAG semantics
- maintainer control-plane commands
- repository-level governance and release policy

## Source Layout

- `python/bijux_cli_py`: Python entrypoints, packaging helpers, and mounted-app
  SDK
- Rust bridge crate: `crates/bijux-cli-python`
- runtime implementation: `crates/bijux-cli`
- DAG runtime implementation: `crates/bijux-dag-cli`

## Reach For Another Surface When

- you need the runtime command semantics themselves: `bijux-cli`
- you need mounted app authoring guidance and contracts:
  `crates/bijux-cli-python/docs/MOUNTED_APPS.md`
- you need repository governance or release automation: `bijux-dev`

## Quick Usage

```bash
python -m pip install bijux-cli
bijux --help
python -m bijux_cli_py --help
```

## DAG Workflow Helpers

The package also ships `bijux_cli_py.dag_sdk` for Python callers that want the
same structured DAG workflow payloads exposed by `bijux-dag --json`.

These helpers delegate to the `bijux-dag` binary. They preserve the CLI JSON
schema instead of wrapping results in a Python-only envelope.

```python
from pathlib import Path

from bijux_cli_py import (
    inspect_dag_run,
    plan_dag_graph,
    query_dag_artifacts,
    run_dag_graph,
    validate_dag_graph,
)

graph = Path("evidence/dag/authoring/examples/hello.dag.json")
run_root = Path("./artifacts/python-dag-runs")

validate = validate_dag_graph(graph)
plan = plan_dag_graph(graph, out=run_root, run_id="python-plan")
run = run_dag_graph(graph, out=run_root, run_id="python-run")
inspect = inspect_dag_run(run_id="python-run", root=run_root)
artifacts = query_dag_artifacts(run_root / "run-python-run")
```

If the runtime binary is not on `PATH`, set `BIJUX_DAG_BIN` to the absolute
path of the `bijux-dag` executable you want the Python helper to call.

## Mounted Python Apps

This package also ships `bijux_cli_py.app_sdk` for Python apps mounted under the
root `bijux` runtime.

It gives app authors:

- `build_python_mount_manifest(...)` for root-mount descriptors with Python
  `module` / `function` entrypoints,
- `CompatibilityWindow` and `compatibility_report(...)` for host-runtime checks,
- `success(...)` and `failure(...)` helpers that emit the same envelope shape as
  the Rust runtime,
- `run_json_app(...)` to keep structured JSON on stdout while redirecting ad hoc
  app logs to stderr.

Typical packaging flow:

```python
from bijux_cli_py.app_sdk import build_python_mount_manifest, run_json_app, success

def main(argv: list[str]):
    return success({"argv": argv}, command=["sample"])

manifest = build_python_mount_manifest(
    namespace="sample",
    display_name="Sample App",
    module="sample_app.cli",
    function="main",
    summary="Sample mounted Python app",
)
```

Then publish the app package with a standard `pyproject.toml`, expose the
callable through your module, and place the generated manifest under
`.bijux/apps/<namespace>.mount.json` or a managed discovery path.

## Related Surfaces

- Runtime crate: [`crates/bijux-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli)
- DAG runtime crate: [`crates/bijux-dag-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli)
- Python bridge crate: [`crates/bijux-cli-python`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli-python)
- Mounted app guide: [`crates/bijux-cli-python/docs/MOUNTED_APPS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli-python/docs/MOUNTED_APPS.md)
- Package changelog: [`crates/bijux-cli-python/CHANGELOG.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli-python/CHANGELOG.md)
- Repository handbook: [CLI handbook](https://bijux.io/bijux-core/bijux-cli/)

## License

Apache-2.0 ([repository LICENSE](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
