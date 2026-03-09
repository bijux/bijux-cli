# Install Channels

The canonical executable is always `bijux`.

## Cargo

```bash
cargo install bijux-cli
cargo install bijux
```

Both commands must expose the same executable contract: `bijux`.

## Pip

```bash
pip install bijux-cli
pip install bijux
```

Both commands must expose the same executable contract: `bijux`.

## Cross-Ecosystem Coexistence

If both ecosystems are installed, runtime selection follows this rule:

1. `BIJUX_BIN` override if present.
2. First `bijux` binary in `PATH` order.

Use `bijux cli paths` and `bijux cli doctor` to inspect and validate active ownership.

## Conflict Rule

No install channel may publish a divergent secondary default executable.
