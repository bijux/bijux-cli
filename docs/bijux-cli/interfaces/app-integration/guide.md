---
title: App Integration Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# App Integration Guide

Mounted apps let `bijux` expose product commands through one root runtime while
keeping each app independently owned.

## Supported Integration Paths

- compiled official app descriptors
- project-local `.bijux/apps/*.mount.json` overrides
- Rust mounted apps built with the `bijux_cli::sdk`
- Python mounted apps built with `bijux_cli_py.app_sdk`

## Minimum Contract

Every app descriptor must declare:

- canonical namespace
- entrypoint kind
- help metadata
- compatibility window when relevant
- stable command output compatible with the root runtime

## Rust Path

Use `ProductMount`, `BijuxApp`, and `BijuxCliHarness` from `bijux-cli` when the
app is owned in Rust and needs in-process contract tests.

## Python Path

Use `bijux_cli_py.app_sdk` when the app is distributed as a Python package.
Prefer callable or module entrypoints that keep stdout reserved for structured
payloads.

## Operator Checks

- `bijux apps list`
- `bijux apps doctor <namespace>`
- `bijux apps which <namespace>`
- `bijux doctor <namespace>`
