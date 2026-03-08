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

## Effects contract for operators

Bijux nodes must declare explicit effects. This contract is used for validation, policy gates, and determinism expectations.

Effects:
- `filesystem`: node reads/writes files in its sandbox
- `env`: node reads environment variables
- `network`: node makes network calls
- `clock`: node reads system time

If `env_allowlist` is non-empty, `env` must be declared.
Policy flags (`--deny-network`, `--deny-env`, `--deny-clock`) reject nodes that declare denied effects.

Adapters declare required effects, and Bijux validates that each node's declared effects cover adapter requirements.
