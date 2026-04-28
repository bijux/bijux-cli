# bijux-cli Python Package

<!-- bijux-core-badges:generated:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)](https://pypi.org/project/bijux-cli/)
[![PyPI](https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi)](https://pypi.org/project/bijux-cli/)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli-python docs](https://img.shields.io/badge/docs-bijux--cli--python-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli-python/)
[![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-cli` is the Python distribution for installing and launching the Bijux
command runtime.

This package provides:

- the `bijux` console entrypoint,
- a native Rust bridge module (`bijux_cli_py._native`) when available,
- a Python facade fallback for portability and compatibility checks.

## What This Package Is

- A packaging and bridge layer for the `bijux` runtime.
- The canonical PyPI surface for `bijux-cli`.
- A compatibility boundary between Python callers and the Rust runtime.

## What This Package Is Not

- It does not define independent runtime semantics.
- It does not publish maintainer control-plane commands.
- It does not replace repository-level governance docs.

## Quick Usage

```bash
python -m pip install bijux-cli
bijux --help
python -m bijux_cli_py --help
```

## Source of Truth

- Runtime crate: [`crates/bijux-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli)
- Python bridge crate: [`crates/bijux-cli-python`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli-python)
- Package changelog: [`crates/bijux-cli-python/CHANGELOG.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli-python/CHANGELOG.md)
- Repository handbook: [CLI handbook](https://bijux.io/bijux-core/bijux-cli/)

## License

Apache-2.0 ([repository LICENSE](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
