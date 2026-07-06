---
title: release-pypi
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-06
---

# release-pypi

`release-pypi.yml` publishes the Python bridge distributions for `bijux-cli`
after the tagged commit passes CI and the target version is not already on
PyPI.

## Trigger

- `push` on tags matching `v*`
- manual `workflow_dispatch`

## Job Shape

- wait for `ci.yml` to pass on the tagged commit
- decide publication with `make gh-release-plan-pypi`
- prepare the release tree
- provision Rust `1.86.0` through `BIJUX_PYPI_RUST_TOOLCHAIN`
- build wheel and source distribution with Maturin
- publish through `make publish-py`

## Next Reads

- [release-crates](release-crates.md)
- [release-github](release-github.md)
- [Release Operations](../operations/release-operations.md)
