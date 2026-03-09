# Install Method Parity

All supported install methods must expose the same behavioral contract for `bijux`.

## Guaranteed Parity

- Same executable name: `bijux`
- Same command grammar and route resolution
- Same exit code mapping
- Same stdout/stderr routing rules
- Same JSON/YAML envelope structure

## Supported Channels

- `cargo install bijux-cli`
- `cargo install bijux` (compatibility alias)
- `pip install bijux-cli`
- `pip install bijux` (compatibility alias)

## Verification

Run these checks after any installation or upgrade:

```bash
bijux version
bijux cli paths
bijux cli doctor
```

