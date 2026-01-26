# API usage

## Purpose
This document guarantees safe embedding of bijux-cli via the Python API.

## Scope
It covers API entrypoints and output behavior only.

## Core Concepts
- API returns data only.
- API does not write to stdout or stderr.

## Invariants
- API purity is enforced when the guard is enabled.

## Execution
```python
from bijux_cli.api import BijuxAPI

api = BijuxAPI()
api.run_sync("status")
```

## Failure Modes
- Invalid command raises a BijuxError.
- API guard violations raise a guard error.

## Design Rationale
- Alternatives: API that prints by default.
- Rejected because it breaks embedding.

## Non-Goals
- Web API usage.

## References
- API purity guard: `src/bijux_cli/api/facade.py`
