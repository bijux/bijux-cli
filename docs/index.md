# Docs Index

High-value documents only. Everything else is supporting detail.

## Current Status
- [Status and gaps](KNOWN_GAPS.md)
- [Plugin runtime law](PLUGIN_RUNTIME_LAW.md)
- [Contributor engineering rules](CONTRIBUTOR_ENGINEERING_RULES.md)
- [Plugin state](plugin_state.md)

## Core Law
- [Architecture index](10-architecture/index.md)
- [Quality and change management](10-architecture/quality-and-change-management.md)
- [Runtime and distribution](10-architecture/runtime-and-distribution.md)
- [Constitution index](constitution/index.md)

## Usage
- [Introduction](01-introduction/index.md)
- [Installation](guides/installation.md)
- [First run](01-introduction/first-run.md)
- [Commands reference](reference/commands.md)
- [Exit codes](reference/exit-codes.md)

## Guides
- [Installation guide](guides/installation.md)
- [Plugin guide](guides/plugins.md)
- [Configuration guide](guides/configuration.md)
- [Development guide](guides/development.md)
- [REPL reference](reference/repl.md)

## Live Checks
- `cargo test --workspace`
- `python3 -m pytest crates/bijux-cli-python/tests/python`
- `bijux dev cli status --format json --no-pretty`
- `bijux dev cli parity --format json --no-pretty`

## Docs Rule
Fewer docs, higher signal. Every long-form doc must explain law or explain change.
Target cap for long-form markdown docs: **60**.
