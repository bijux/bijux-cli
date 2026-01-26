# API usage

Goal: embed bijux via the Python API.

```python
from bijux_cli.api import BijuxAPI

api = BijuxAPI()
api.run_sync("status")
```

Notes:

- API returns data only
- No stdout/stderr when API guard is enabled
