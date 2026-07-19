# bijux-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/)
<!-- bijux-core-badges:generated:end -->

`bijux-cli` is the public Rust package behind the `bijux` command runtime.

It is the source of truth for command semantics shared by the native binary,
the Python distribution, and the in-process SDK surfaces used by mounted apps
and integration tests.

Use it when you want the `bijux` runtime itself, or when you want to embed
mounted app behavior against the same envelopes, exit codes, and routing rules
that the installed command uses.

Install the end-user command with either of the public distribution paths:

```bash
cargo install bijux-cli
python -m pip install bijux-cli
```

Then inspect the supported runtime surface with:

```bash
bijux --help
bijux doctor
bijux apps --help
```

## What It Provides

- Own command parsing, normalization, registry lookup, and execution.
- Own runtime-facing state behavior for config, history, memory, install diagnostics, plugins, and the REPL.
- Expose read-only query APIs used by maintainer tooling.
- Do not assemble maintainer reports; `bijux-dev-cli` owns that surface directly.

## Source Layout

- [`src/api`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/api): stable entrypoints used by the binary, tests, and the Python bridge.
- [`src/bootstrap`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/bootstrap): process wiring and exit-code handling.
- [`src/contracts`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/contracts): durable command, envelope, config, plugin, and query types.
- [`src/features`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/features): domain implementations for config, diagnostics, history, install, memory, and plugins.
- [`src/infrastructure`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/infrastructure): filesystem, process, environment, and state-store adapters.
- [`src/interface`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/interface): CLI and REPL surfaces.
- [`src/kernel`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/kernel): execution pipeline and policy resolution.
- [`src/routing`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/routing): command catalog, parser, and registry.
- [`src/shared`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/shared): small cross-cutting helpers.

## Reach For Another Surface When

- you need Python packaging, interpreter diagnostics, or mounted Python app
  distribution: `bijux-cli-python`
- you need repository diagnostics, governance reports, or release proof:
  `bijux-dev`
- you need DAG graph execution rather than the root runtime:
  `bijux-dag-*`

## Runtime Rules

- Commands are parsed and normalized before execution.
- Help, envelopes, and output formatting stay deterministic across repeated runs.
- Maintainer commands stay outside the runtime binary; this crate does not parse or execute `bijux-dev-cli` surfaces.
- The process entrypoint stays thin: decode argv, call the runtime, write streams, map exit codes.

## Mounted App SDK

The crate-native SDK under
[`src/sdk`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/sdk)
uses the same routing context and output envelope as the installed command.
`ProductMount` declares the mount, `BijuxApp` handles routed calls, and
`BijuxCliHarness` exercises the boundary without spawning a process.

```rust
use bijux_cli::sdk::ProductMount;

let mount = ProductMount::new("hello")?
    .binary("bijux-hello")
    .summary("Hello application");
```

Python-mounted apps use the same descriptor contract. See the
[mounted Python app guide](../bijux-cli-python/docs/MOUNTED_APPS.md) for interpreter discovery,
manifest placement, compatibility checks, and packaging.

## Operator References

| Question | Authority |
| --- | --- |
| which commands and output contracts are supported? | [CLI Surface](../../docs/bijux-cli/interfaces/cli-surface.md) |
| how are global, profile, project, and environment values resolved? | [Configuration Surface](../../docs/bijux-cli/interfaces/configuration-surface.md) |
| how do I diagnose paths, routing, plugins, Python, or mounted apps? | [Diagnostics Guide](../../docs/bijux-cli/operations/diagnostics-guide.md) |
| which generated keys and scopes exist? | [Generated Configuration Reference](../../docs/bijux-cli/interfaces/generated-config-reference.md) |

## Tests

- [`tests/architecture.rs`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli/tests/architecture.rs): boundary and ownership checks.
- [`tests/integration.rs`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli/tests/integration.rs): command behavior, parity, resilience, and REPL coverage.
- [`tests/routing.rs`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli/tests/routing.rs): parser, registry, schema, and routing law coverage.
- [`tests/data/fixtures`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/tests/data/fixtures) and [`tests/data/golden`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/tests/data/golden): stable fixtures and snapshots.

## Release References

- Repository handbook: [CLI handbook](https://bijux.io/bijux-core/bijux-cli/)
- Crate changelog: [`crates/bijux-cli/CHANGELOG.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli/CHANGELOG.md)
- Root release log: [`CHANGELOG.md`](https://github.com/bijux/bijux-core/blob/main/CHANGELOG.md)
- Security policy: [`SECURITY.md`](https://github.com/bijux/bijux-core/blob/main/SECURITY.md)
