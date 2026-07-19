---
title: Python Bridge Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Python Distribution Bridge

`bijux-cli-python` packages the `bijux` runtime for Python installation and
provides helpers for Python-mounted apps. It is a distribution and process
boundary, not an independent implementation of CLI semantics.

## Owned Surface

- wheel metadata and supported interpreter declarations
- `python -m bijux_cli_py` and Python console-script entrypoints
- native-extension loading when the installed wheel includes one
- `bijux_cli_py.app_sdk` helpers for mounted Python apps
- conversion between Python values and the runtime output envelope

Routing, configuration precedence, plugin policy, and output semantics remain
owned by `bijux-cli`. A Python entrypoint must produce the same stable result as
the native entrypoint for equivalent input.

## Diagnosis

```bash
python -c 'import bijux_cli_py; print(bijux_cli_py.__file__)'
python -m bijux_cli_py version
bijux doctor python
bijux apps doctor <python-mounted-app>
```

An import failure is a packaging or interpreter compatibility problem. A
successful import followed by an app-doctor failure points to mount metadata,
the app executable, or the app protocol. Diagnose those cases separately.

Mounted apps must keep stdout available for their structured response and send
diagnostics to stderr. They must not depend on repository source paths or the
development virtual environment. Validate wheel behavior from an isolated
installation when release packaging is in scope.

## Compatibility

The Python package version follows the workspace release line, but wheel tags
and interpreter support are Python-distribution concerns. Changes to bridge
conversion require parity tests in both Rust and Python; changes to the public
app protocol require the owning CLI contract and package documentation to
change together.
