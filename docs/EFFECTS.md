# Effects Contract

Bijux nodes must declare explicit effects. Effects are used for validation, policy gates,
and determinism expectations.

Effects:
- `filesystem`: node reads/writes files in its sandbox
- `env`: node reads environment variables
- `network`: node makes network calls
- `clock`: node reads system time

Rules:
- Shell and container nodes must declare `filesystem` at minimum.
- If `env_allowlist` is non-empty, `env` must be declared.
- Policy flags (`--deny-network`, `--deny-env`, `--deny-clock`) reject nodes that
  declare denied effects.

Adapters declare required effects and bijux validates that a node's declared effects
cover those requirements.
