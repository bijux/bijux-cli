# Policy Gates

Bijux provides policy flags to enforce organization rules without changing DAGs.
These gates are evaluated before node execution.

## Flags

- `--deny-network` blocks nodes that declare the `network` effect.
- `--deny-env` blocks nodes that declare the `env` effect.
- `--deny-clock` blocks nodes that declare the `clock` effect.

These are strict: if a node declares an effect that the policy denies, validation
(or runtime) fails with a policy error.

## Examples

### Deny all network access

```
bijux-dag run dag.json --out runs/ --deny-network
```

### Enforce an env allowlist

Shell nodes must declare `env` if they use `env_allowlist`:

```json
{
  "id": "build",
  "kind": "shell",
  "inputs": ["src"],
  "outputs": ["bin"],
  "params": {"argv": ["make"]},
  "effects": ["filesystem", "env"],
  "env_allowlist": ["PATH", "MAKEFLAGS"]
}
```

## Notes

- Policy is recorded in `manifest.json` under `policy`.
- These gates are a policy layer; they do not attempt to sandbox the OS.
