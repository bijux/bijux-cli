# bijux-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
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

`bijux-cli` now exposes a crate-native SDK for mounted Rust apps under [`src/sdk`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli/src/sdk).

Core surfaces:

- `ProductMount`: high-level mounted-app builder for binary, Python-module, console-script, plugin-process, and embedded-Rust entrypoints
- `BijuxApp`: trait for routed app implementations
- `CommandContext`: stable execution context for mounted handlers
- `CommandResult`: root-compatible result envelope with explicit stream policy
- `BijuxCliHarness`: in-process harness for mounted app tests
- `SnapshotHelper`: stable rendering helper for app-level snapshot contracts

Minimal example:

```rust
use bijux_cli::sdk::{
    BijuxApp, CommandContext, CommandResult, OutputEnvelopeHelper, ProductMount,
};

struct HelloApp;

impl BijuxApp for HelloApp {
    fn mount(&self) -> ProductMount {
        ProductMount::new("hello")
            .expect("namespace")
            .binary("bijux-hello")
            .summary("Minimal hello app")
    }

    fn route(&self, argv: &[String], ctx: &CommandContext) -> CommandResult {
        let command = ctx.command_path(&["status"]).expect("command path");
        CommandResult::success(
            OutputEnvelopeHelper::success(
                command,
                serde_json::json!({ "status": "ok", "argv": argv }),
                "1970-01-01T00:00:00Z",
            )
            .expect("success envelope"),
        )
    }
}
```

Python-mounted apps use the same descriptor contract. The runtime validates
`python_module` entrypoints with optional callable fields, resolves a concrete
interpreter from the active environment or project `.venv`, and exposes
`bijux apps doctor <namespace>` for import, version, and callable diagnostics.
The companion package guide lives at
[`crates/bijux-cli-python/docs/MOUNTED_APPS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-cli-python/docs/MOUNTED_APPS.md).

## Runtime Diagnostics

The root `doctor` surface now acts as the operator-facing runtime diagnostic entrypoint:

- `bijux doctor`: unified install, state-path, plugin, app-mount, routing, and shim health
- `bijux doctor --bundle`: export a bug-report-ready runtime bundle under `./artifacts`
- `bijux doctor paths`: resolved state files plus read/write diagnostics
- `bijux doctor python`: Python bridge interpreter, import, and console-script diagnostics
- `bijux doctor routing`: canonical built-in routes, aliases, and namespace inventory
- `bijux doctor shims`: deprecated alias-binary detection without flagging declared product binaries such as `bijux-dag`
- `bijux doctor <app>`: focused official app discovery and runtime diagnostics

## Layered Configuration

The config surface now supports a stronger operator workflow than plain key-value
mutation:

- `bijux config schema [scope]`: inspect the built-in config registry for `cli`,
  `dag`, and mounted-app scopes
- `bijux config docs [scope]`: generate a markdown reference from the same
  built-in schema registry
- `bijux config validate [--profile name]`: validate effective config across the
  global env file, named profile overlays, project `.bijux/config.{toml,json}`,
  and environment overrides
- `bijux config explain KEY`: show the effective source chain for one key with
  secret-aware redaction
- `bijux config repair`: recover malformed global env state and write a backup
- `bijux config export/load --portable`: round-trip a logical-key JSON bundle
  instead of only dotenv-style env files

The generated handbook reference lives at
[`docs/bijux-cli/interfaces/config/generated-reference.md`](https://github.com/bijux/bijux-core/blob/main/docs/bijux-cli/interfaces/config/generated-reference.md).

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
