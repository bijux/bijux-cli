---
title: bijux-cli-python Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-cli-python

`bijux-cli-python` is the Python distribution and native bridge for installing
and launching the Bijux command runtime. It is the packaging boundary between
Python callers and the Rust runtime, not a second source of runtime truth.

Use this page when the issue is about PyPI packaging, launcher behavior,
bridge compatibility, or Python-facing parity with the native binary.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| packaging | Python distribution metadata, entrypoints, and install surface |
| bridge | native bindings, conversion layer, compatibility checks, fallback facade |
| release parity | alignment between `bijux` binary behavior and Python launcher behavior |
| boundary | does not redefine runtime semantics already owned by `bijux-cli` |

## Source Layout

- `crates/bijux-cli-python/pyproject.toml`
- `crates/bijux-cli-python/python/bijux_cli_py`
- `crates/bijux-cli-python/src/lib.rs`
- `crates/bijux-cli-python/src/bindings.rs`
- `crates/bijux-cli-python/src/conversions.rs`
- `crates/bijux-cli-python/src/compatibility.rs`
- `crates/bijux-cli-python/tests`

## Open Next

- open the [CLI Handbook](../../index.md) for product-level runtime behavior
- open [`bijux-cli`](../bijux-cli/index.md) when the question is native runtime
  ownership rather than distribution or bridge mechanics
- open the [Repository Handbook](../../../bijux-core/index.md) when the issue
  touches release governance, workspace policy, or cross-program ownership

## Code Anchors

- `crates/bijux-cli-python/README.md`
- `crates/bijux-cli-python/Cargo.toml`
- `crates/bijux-cli-python/pyproject.toml`
- `crates/bijux-cli-python/tests/python/test_runtime_parity.py`
- `crates/bijux-cli-python/tests/runtime_entrypoint_unity.rs`

## Review Lens

- Python-facing entrypoints should preserve runtime parity instead of drifting into custom behavior
- package metadata should route readers back to the CLI handbook rather than duplicating it
- release changes should keep bridge compatibility explicit and test-backed
