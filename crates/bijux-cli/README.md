# bijux-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-cli` is the Rust runtime crate behind the `bijux` executable.

It is the public command runtime product in `v0.3.5` and the source of truth
for runtime command semantics shared by the native binary and Python bridge.

## Scope

- Own command parsing, normalization, registry lookup, and execution.
- Own runtime-facing state behavior for config, history, memory, install diagnostics, plugins, and the REPL.
- Expose read-only query APIs used by maintainer tooling.
- Do not assemble maintainer reports; `bijux-dev-cli` owns that surface directly.

## Source Layout

- `src/api`: stable entrypoints used by the binary, tests, and the Python bridge.
- `src/bootstrap`: process wiring and exit-code handling.
- `src/contracts`: durable command, envelope, config, plugin, and query types.
- `src/features`: domain implementations for config, diagnostics, history, install, memory, and plugins.
- `src/infrastructure`: filesystem, process, environment, and state-store adapters.
- `src/interface`: CLI and REPL surfaces.
- `src/kernel`: execution pipeline and policy resolution.
- `src/routing`: command catalog, parser, and registry.
- `src/shared`: small cross-cutting helpers.

## Runtime Rules

- Commands are parsed and normalized before execution.
- Help, envelopes, and output formatting stay deterministic across repeated runs.
- Maintainer commands stay outside the runtime binary; this crate does not parse or execute `bijux-dev-cli` surfaces.
- The process entrypoint stays thin: decode argv, call the runtime, write streams, map exit codes.

## Tests

- `tests/architecture.rs`: boundary and ownership checks.
- `tests/integration.rs`: command behavior, parity, resilience, and REPL coverage.
- `tests/routing.rs`: parser, registry, schema, and routing law coverage.
- `tests/data/fixtures` and `tests/data/golden`: stable fixtures and snapshots.

## Release References

- Repository handbook: `docs/bijux-cli/`
- Crate changelog: `crates/bijux-cli/CHANGELOG.md`
- Root release log: `CHANGELOG.md`
- Security policy: `SECURITY.md`
