# Security Model

## Shell Executor
- Runs in a node-specific sandbox directory.
- Working directory is forced to the node sandbox dir.
- Only declared environment variables are passed.
- Command must be an argv list; raw shell strings are forbidden.

## Effects
Nodes must declare their effects:
- `filesystem`
- `network`
- `env`

Shell nodes must explicitly declare effects.
