# API usage

## Purpose
This document tells you how to embed bijux-cli without IO side effects.

## Scope
It covers API entrypoints and error behavior only.

## What problem this solves
Printing from an API call corrupts host applications.
The API guard prevents that.

## Why you should care
If you embed bijux-cli, you need pure return values and predictable errors.

## What confusion this removes
It removes doubt about whether the API writes to stdout or stderr.

## Guarantees
Bijux guarantees:
1. API calls return data only.
2. API guard violations raise errors.

## How to Think About This
Treat the API as a function library, not a CLI wrapper.

## Common Misunderstandings
- "API calls behave like CLI commands." They do not.

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
We deliberately chose purity to make embedding safe.
Why not allow printing? It breaks host process output.

## Non-Goals
- Web API usage.

## References
- API purity guard: `src/bijux_cli/api/facade.py`
