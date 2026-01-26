# Writing plugins

This guide shows a minimal plugin and required invariants.

## Minimal plugin structure

```
my_plugin/
  plugin.py
  plugin.toml
```

## Example plugin.py

```python
import typer

app = typer.Typer()

@app.command()
def hello() -> None:
    print("hello")
```

## Required metadata

- name
- version
- cli compatibility

## Testing

- install -> list -> info -> uninstall
- exit codes are stable
- no leftover files
