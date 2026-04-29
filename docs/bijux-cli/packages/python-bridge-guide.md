---
title: Python Bridge Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Python Bridge Guide

`bijux-cli-python` is the packaging and runtime bridge for Python-distributed
CLI entrypoints.

## What It Owns

- `python -m bijux_cli_py ...`
- Python wheel metadata and console-script entrypoints
- Python mounted-app helpers in `bijux_cli_py.app_sdk`
- parity expectations between Rust runtime and Python launch paths

## What To Validate

- a supported Python interpreter is available
- `bijux_cli_py` imports cleanly
- the expected console script is present when wheel installs are in scope
- Python-mounted apps keep stdout reserved for structured payloads

## Runtime Checks

- `bijux doctor python`
- `bijux apps doctor <python-mounted-app>`
- `python -m bijux_cli_py version`
