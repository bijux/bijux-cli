# Workflows

Config + status workflow:

```bash
bijux config set mode strict
bijux status --format json
bijux config unset mode
```

Config cycle:

```bash
bijux config set foo=bar
bijux config get foo
bijux config list --format json
bijux config unset foo
```
