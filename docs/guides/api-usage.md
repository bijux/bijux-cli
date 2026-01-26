# API Usage

## Purpose
This guide explains how to use the bijux-cli Python API in a way that preserves the same guarantees as the CLI. It exists to prevent accidental side effects when embedding bijux-cli in applications.

## Scope
It covers the core API entry points and expected behavior. It does not describe the HTTP API or the CLI command syntax.

## Using the API
The API returns data structures and does not perform output routing or process exits. This keeps the API usable in headless environments.

## Example
Use the API to run a command and inspect the result directly.

```python
from bijux_cli.api.facade import BijuxAPI

api = BijuxAPI()
result = api.run_sync("status")
print(result)
```

## Why This Matters
Keeping the API side-effect free ensures that calling code remains in control of output, logging, and exit behavior. This is critical for embedding in servers or long-running processes.

## References
- API facade: `src/bijux_cli/api/facade.py`
