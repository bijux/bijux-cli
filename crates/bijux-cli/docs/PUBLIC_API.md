# `bijux-cli` Public API

The supported Rust integration surface is intentionally narrower than the
crate's implementation. Consumers should enter through `api`, `contracts`, or
`sdk`; private modules may change without a downstream compatibility promise.

## Runtime Facade

`bijux_cli::api::runtime` exposes:

- `run_app`, which executes an explicit argument vector and returns
  `AppRunResult`;
- `run_cli_from_env`, which owns process-style argument acquisition and
  stream/exit integration.

Embedded callers should prefer `run_app`. They receive stdout, stderr, and the
exit code as data instead of surrendering process control. Arguments are
expected to carry the executable name, matching native invocation semantics.

## Query And Support APIs

The remaining `api` modules provide narrow owned views:

| Module | Intended use |
| --- | --- |
| `config` | validate runtime configuration |
| `diagnostics` | inspect paths and runtime diagnostic records |
| `install` | resolve compatibility paths, locks, and migrations |
| `output` | render governed output and error shapes |
| `parser` and `routing` | inspect canonical parsing and route authorities |
| `plugins` | list plugins and inspect load-time diagnostics |
| `repl` | use the owned REPL parsing and session surfaces |
| `telemetry` | consume bounded invocation telemetry |
| `version` | obtain runtime version and build provenance |

These modules re-export owned behavior. Consumers should not duplicate the
underlying implementation from private source paths.

## Contract Types

`bijux_cli::contracts` is the authority for serialized and cross-package data.
Important families include:

- command and namespace identity;
- configuration values, provenance, mutations, and schema registries;
- output, error, warning, and command envelopes;
- global execution flags, policy, output modes, and `ExitCode`;
- plugin manifests, capabilities, trust classes, and lifecycle state;
- mounted-product descriptors and compatibility windows;
- schema constructors used by generated references and validators.

Adding a field to a serialized contract requires checking schema compatibility,
fixture expectations, Python conversion behavior, and command output parity.
Do not treat a Rust default as permission to change the wire contract silently.

## Mounted-Application SDK

`bijux_cli::sdk` provides the higher-level application boundary:

- `ProductMount` and compatibility declarations describe a mounted product;
- `CommandContext` carries the routed invocation context;
- `BijuxApp` implements application dispatch;
- `CommandResult`, `CommandEnvelope`, and render configuration preserve root
  output semantics;
- `BijuxCliHarness` runs application behavior without process spawning.

Use the SDK for application composition. Use `run_app` when the requirement is
to invoke the root runtime exactly as a command would.

## Compatibility Rules

- Public names and serialized fields require release-note review.
- Exit-code changes are behavioral compatibility changes.
- Schemas and generated references must agree with their Rust authorities.
- Python bindings may translate representation but cannot redefine meaning.
- Repository-private modules are not imported by downstream crates.
- A new public re-export needs a named consumer and focused contract tests.

## Verification

The API surface is covered across:

- `tests/architecture.rs` for allowed exports and package boundaries;
- `tests/routing.rs` for parser and route facade behavior;
- `tests/integration.rs` for runtime results and command contracts;
- `src/kernel/tests.rs` for kernel API laws;
- `crates/bijux-cli-python/tests/` for bridge and binary parity.

For API documentation and compile-time validation:

```bash
cargo doc --locked -p bijux-cli --no-deps
cargo test --locked -p bijux-cli --test architecture
```
