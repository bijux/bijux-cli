# Extending the CLI

Add a command by creating a Typer command and registering it.

Steps:

1. Create a command module
2. Register it in the command registry
3. Add tests for output and exit behavior

Example (simplified):

```python
import typer

app = typer.Typer()

@app.command()
def ping() -> None:
    print("pong")
```

Keep parsing side-effect free and use exit intents for errors.
