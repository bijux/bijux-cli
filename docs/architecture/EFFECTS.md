# Effects

Effects declare side effects required by a node or adapter:
- `filesystem`
- `network`
- `env`
- `clock`

## Policy
- Shell nodes must include `filesystem`.
- `env_allowlist` requires `env`.
- `--deny-network` rejects nodes declaring `network`.
- Retries with `network` or `clock` require `inputs.random_seed` or `nondeterminism_allowed`.
