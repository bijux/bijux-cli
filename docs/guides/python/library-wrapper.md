# Using bijux-cli as a Python Library Wrapper

`bijux-cli` exposes a Python facade in `bijux_cli_py` for embedding CLI execution in Python code.

## Common APIs

- `version()`
- `command_tree_introspection()`
- `execution_facade(argv)`
- `execution_facade_with_status(argv)`

## Example

```python
from bijux_cli_py import execution_facade_with_status

result = execution_facade_with_status(["status", "--format", "json"])
if result.exit_code == 0:
    print(result.stdout)
else:
    print(result.stderr)
```
