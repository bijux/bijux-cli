---
title: bijux-cli-python Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-08
---

# bijux-cli-python

<!-- bijux-core-badges:generated:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)](https://pypi.org/project/bijux-cli/)
[![PyPI](https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi)](https://pypi.org/project/bijux-cli/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli-python docs](https://img.shields.io/badge/docs-bijux--cli--python-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli-python/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-cli-python` is the Python distribution and native bridge for installing
and launching the Bijux command runtime. It also owns the Python DAG helper
surface that delegates graph operations to `bijux-dag`. It remains a packaging
and delegation boundary between Python callers and Rust runtimes, not a second
source of runtime truth.

Use this page when the issue is about PyPI packaging, launcher behavior,
Python-facing parity, interpreter diagnostics, or the bridge rules between
Python callers and the Rust runtimes.

## Reach For This Package When

- `pip install bijux-cli` succeeds but the console script or module entrypoint
  behaves incorrectly
- Python and native entrypoints disagree on output, flags, or runtime behavior
- mounted Python apps fail import, callable resolution, or interpreter checks
- a Python caller needs DAG helper access without inventing a second DAG
  protocol

## What It Owns

| Surface | Ownership |
| --- | --- |
| packaging | Python distribution metadata, entrypoints, and install surface |
| bridge | native bindings, conversion layer, compatibility checks, fallback facade |
| release parity | alignment between `bijux` binary behavior and Python launcher behavior |
| DAG helpers | Python load/validate/plan/run/inspect/query helpers that preserve `bijux-dag` JSON payloads |
| boundary | does not redefine runtime semantics already owned by `bijux-cli` or `bijux-dag-cli` |

## What It Must Preserve

- the same command semantics as the native `bijux` runtime
- explicit compatibility checks instead of silent drift between launcher paths
- DAG helper payloads that stay aligned with the retained `bijux-dag` JSON
  contracts

## Source Layout

- `crates/bijux-cli-python/pyproject.toml`
- `crates/bijux-cli-python/python/bijux_cli_py`
- `crates/bijux-cli-python/python/bijux_cli_py/dag_sdk.py`
- `crates/bijux-cli-python/src/lib.rs`
- `crates/bijux-cli-python/src/bindings.rs`
- `crates/bijux-cli-python/src/conversions.rs`
- `crates/bijux-cli-python/src/compatibility.rs`
- `crates/bijux-cli-python/tests`

## Practical Starting Points

- open the [CLI Handbook](../index.md) for product-level runtime behavior
- open [`bijux-cli`](bijux-cli.md) when the question is native runtime
  ownership rather than distribution or bridge mechanics
- open [Python Bridge Guide](python-bridge-guide.md) when you want the shortest
  route to the bridge-specific validation story
- open [`bijux-dag`](../../bijux-dag/index.md) when the question is DAG runtime
  semantics rather than Python delegation
- open the [Repository Handbook](../../bijux-core/index.md) when the issue
  touches release governance, workspace policy, or cross-program ownership

## Code Anchors

- `crates/bijux-cli-python/README.md`
- `crates/bijux-cli-python/Cargo.toml`
- `crates/bijux-cli-python/pyproject.toml`
- `crates/bijux-cli-python/tests/python/test_runtime_parity.py`
- `crates/bijux-cli-python/tests/runtime_entrypoint_unity.rs`
- `crates/bijux-cli-python/tests/python/test_dag_sdk_transport.py`
- `crates/bijux-cli-python/tests/python/test_dag_sdk_workflows.py`

## Review Focus

- Python-facing entrypoints should preserve runtime parity instead of drifting into custom behavior
- DAG helpers should return the same structured payloads a caller would receive
  from `bijux-dag --json`
- package metadata should route readers back to the CLI handbook rather than duplicating it
- release changes should keep bridge compatibility explicit and test-backed
