# Adapters

Adapters are the execution backends for nodes. Each adapter exposes:

- `adapter_id` (string)
- `adapter_version` (string)
- `required_effects` (filesystem/env/network/clock)

At runtime, bijux-dag verifies that a node's declared `effects` cover the adapter's
required effects. Adapters must be deterministic for identical inputs and must
write outputs only to the node sandbox.

## Built-in Adapters

- `const` v0.1
  - Effects required: none
  - Writes `outputs/value.json` from `params.value`.
- `shell` v0.1
  - Effects required: `filesystem`
  - Executes an argv list in the node's `work/` directory with a cleared env.

## Example: Shell Node

```json
{
  "id": "compile",
  "kind": "shell",
  "inputs": ["src"],
  "outputs": ["bin"],
  "params": {
    "argv": ["/usr/bin/cc", "-o", "out.bin", "in.c"]
  },
  "effects": ["filesystem", "env"],
  "env_allowlist": ["CC", "CFLAGS"]
}
```

## Adding a New Adapter

1. Implement the `Adapter` trait (see `crates/bijux_dag_runtime/src/adapter.rs`).
2. Declare required effects explicitly.
3. Ensure outputs are written only under `nodes/<id>/outputs/`.
4. Add adapter metadata to traces and manifest via runtime registration.
