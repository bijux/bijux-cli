# Config Crate Ownership

This document defines stable ownership boundaries for Rust config behavior.

## Ownership decisions

- Config contracts: `bijux-cli-routing` (`contracts` module)
- Config storage and path compatibility: `bijux-cli-install`
- Config command routing and identity: `bijux-cli-routing`
- Config execution semantics and command handlers: `bijux-cli-core`
- Config output rendering: `bijux-cli-output`
- Process bootstrap only: `bijux-cli-bin`

## Boundary rules

- `bijux-cli-bin` must not contain config parsing, validation, migration, or file-write logic.
- `bijux-cli-routing` must parse argv and provide route identity only.
- `bijux-cli-core` executes config commands through one app entrypoint and config service API.
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

- Durable and cross-crate config source contracts remain in `bijux-cli-routing` (`contracts` module).
- Storage compatibility and path resolution remain in `bijux-cli-install`.
- Command handler internals remain in `bijux-cli-core` behind config service/repository boundaries.

## Enforced architecture tests

- `crates/bijux-cli-core/tests/config_architecture_boundaries.rs`
  - verifies `bin` stays free of config business logic
  - verifies config storage stays free of output formatting logic
