# Config Crate Ownership

This document defines stable ownership boundaries for Rust config behavior.

## Ownership decisions

- Config contracts: `bijux-cli` (`contracts` module)
- Config storage and path compatibility: `bijux-cli::install`
- Config command routing and identity: `bijux-cli`
- Config execution semantics and command handlers: `bijux-cli`
- Config output rendering: `bijux-cli-output`
- Process bootstrap only: `bijux-cli`

## Boundary rules

- `bijux-cli` must not contain config parsing, validation, migration, or file-write logic.
- `bijux-cli` must parse argv and provide route identity only.
- `bijux-cli` executes config commands through one app entrypoint and config service API.
- Config file persistence must remain in storage/repository components, separate from output concerns.

## Config API shape in core

- One config command entrypoint: `core::config::execute_config_command(...)`
- One config service trait: `ConfigService`
- One config repository trait: `ConfigRepository`
- One config path provider trait: `ConfigPathProvider`
- Explicit modules:
  - `config::validation`
  - `config::serialization`
  - `config::error`
  - `config::storage`
  - `config::service`

## Domain placement

- Durable and cross-crate config source contracts remain in `bijux-cli` (`contracts` module).
- Storage compatibility and path resolution remain in `bijux-cli::install`.
- Command handler internals remain in `bijux-cli` behind config service/repository boundaries.

## Enforced architecture tests

- `crates/bijux-cli/tests/config_architecture_boundaries.rs`
  - verifies `bin` stays free of config business logic
  - verifies config storage stays free of output formatting logic
