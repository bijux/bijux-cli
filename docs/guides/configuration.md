# Configuration

Goal: configure bijux for CI.

```bash
export BIJUXCLI_FORMAT=json
export BIJUXCLI_LOG_LEVEL=info
```

Set config file values:

```bash
bijux config set ci.enabled true
bijux config get ci.enabled
```
