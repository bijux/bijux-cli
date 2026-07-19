# `bijux-cli` Contracts

`bijux-cli` owns the public `bijux` command runtime. It is the semantic
authority shared by the native executable, embedded Rust callers, mounted
applications, and the Python distribution. This page defines the package
boundary; the public handbook explains how operators use it.

## Owned Surface

The crate owns:

- command parsing, canonical route selection, and reserved namespace policy;
- configuration resolution consumed by command execution;
- plugin and mounted-application discovery and dispatch;
- history, memory, diagnostics, and interactive runtime behavior;
- stable output and error envelopes;
- the Rust mounted-application SDK and in-process command harness;
- process exit-code mapping for the `bijux` executable.

The `api`, `contracts`, and `sdk` modules are public integration boundaries.
Bootstrap, feature, infrastructure, interface, kernel, routing, and shared
modules are implementation boundaries even when repository tests exercise
them directly.

## Inputs And Outputs

| Input | Contract |
| --- | --- |
| command arguments | normalized once before route selection; aliases cannot create different semantics |
| configuration | precedence and provenance remain observable rather than being silently flattened |
| plugin or mount metadata | validated against owned schemas before any external process is launched |
| filesystem state | accessed through infrastructure boundaries and reported with actionable path context |
| embedded invocation | uses the same routing and envelope behavior as the installed command |

Every machine-readable command result uses a governed output or error
envelope. Human rendering may add explanation, but it cannot change status,
discard structured diagnostics, or claim success for a failed operation.

## Effect Boundary

Parsing, normalization, schema validation, and route lookup should remain pure.
Filesystem access, environment reads, subprocess execution, terminal IO, and
state mutation belong behind infrastructure or interface boundaries.

The binary entrypoint must stay thin: decode arguments, invoke the runtime,
write the selected streams, and map the runtime result to an exit code.

## Invariants

- Native, embedded, and Python-launched invocations preserve command meaning.
- Help, schema, registry lookup, and execution agree on canonical routes.
- Product mounts cannot claim reserved root namespaces.
- JSON output is deterministic for the same command and controlled state.
- A plugin failure remains distinguishable from routing, configuration, and
  host-runtime failures.
- Maintainer commands and DAG execution semantics do not enter this crate.

## Dependency Direction

`bijux-cli-python` may depend on this crate and translate its types.
`bijux-dev` may inspect its public query surfaces. This crate must not depend on
the Python bridge, DAG packages, or maintainer control plane.

Shared command schemas live under `contracts/`; repository policy and report
generation remain outside the runtime package.

## Failure Contract

The runtime refuses ambiguous routes, malformed manifests, incompatible plugin
metadata, invalid configuration, and unavailable required state. It must not
fall back to an unrelated command, silently discard invalid fields, or replace
a structured error with an empty successful result.

External execution failures preserve the selected executable, status class,
and available stderr context without exposing secrets.

## Governing Schemas

- `contracts/official_product_namespace_registry.json`
- `contracts/product_mount_metadata_contract.json`
- `contracts/schemas/error-envelope-v1.schema.json`
- `contracts/schemas/output-envelope-v1.schema.json`
- `contracts/schemas/plugin-manifest-v2.schema.json`

## Verification

| Claim | Required evidence |
| --- | --- |
| ownership and dependency direction | `crates/bijux-cli/tests/architecture.rs` |
| parser, registry, and route behavior | `crates/bijux-cli/tests/routing.rs` |
| command, plugin, REPL, and envelope behavior | focused suites in `crates/bijux-cli/tests/integration.rs` |
| Python/native parity | owning tests under `crates/bijux-cli-python/tests/` |

Run focused Rust package checks from the repository root:

```bash
cargo test --locked -p bijux-cli --test architecture --test routing
```

A public route, envelope, exit-code, plugin, or mount compatibility change must
update its schema or registry authority, focused tests, package README, and
public handbook in the same change.
