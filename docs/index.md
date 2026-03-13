# Docs Index

High-value documents only. Everything else is supporting detail.

## Current Status
- [Status and gaps](KNOWN_GAPS.md)
- [Plugin runtime law](PLUGIN_RUNTIME_LAW.md)
- [Development guide](05-development/index.md)
- [Plugin state](plugin_state.md)

## Core Law
- [Architecture index](04-architecture/index.md)
- [Quality and change management](04-architecture/quality-and-change-management.md)
- [Runtime and distribution](04-architecture/runtime-and-distribution.md)
- [Contracts index](07-contracts/index.md)

## Usage
- [Introduction](01-introduction/index.md)
- [Getting started](02-getting-started/index.md)
- [Install and verify](02-getting-started/install-and-verify.md)
- [Reference index](06-reference/index.md)
- [Command surface](06-reference/command-surface.md)

## Guides
- [User guide](03-user-guide/index.md)
- [Everyday commands](03-user-guide/everyday-commands.md)
- [Configuration and output](03-user-guide/configuration-and-output.md)
- [Plugins and extensions](03-user-guide/plugins-and-extensions.md)
- [Installation guide](guides/installation.md)
- [Development guide](05-development/index.md)
- [Integrations and routed runtimes](06-reference/integrations-and-routed-runtimes.md)

## Live Checks
- `cargo test --workspace`
- `python3 -m pytest crates/bijux-cli-python/tests/python`
- `bijux dev cli status --format json --no-pretty`
- `bijux dev cli parity --format json --no-pretty`

## Docs Rule
Fewer docs, higher signal. Every long-form doc must explain law or explain change.
Target cap for long-form markdown docs: **60**.
