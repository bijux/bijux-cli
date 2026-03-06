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
  - Writes `params.value` to the declared output file.
- `shell` v0.1
  - Effects required: `filesystem`
  - Executes an argv list in the node's `work/` directory with a cleared env.
- `container` v0.1
  - Effects required: `filesystem`
  - Runs a container with the node sandbox mounted at `/bijux/node`.
  - Uses `container.engine` (`docker` or `podman`) and `container.argv`.

## Example: Shell Node

```json
{
  "id": "compile",
  "kind": "shell",
  "inputs": ["src"],
  "outputs": [{"name": "bin", "path": "bin"}],
  "params": {
    "argv": ["/usr/bin/cc", "-o", "out.bin", "in.c"]
  },
  "effects": ["filesystem", "env"],
  "env_allowlist": ["CC", "CFLAGS"]
}
```

## Adding a New Adapter

1. Implement the `Adapter` trait (see `crates/bijux-dag-runtime/src/adapter.rs`).
2. Declare required effects explicitly.
3. Ensure outputs are written only under `nodes/<id>/outputs/`.
4. Add adapter metadata to traces and manifest via runtime registration.

## External Adapter Protocol (v0.1)

If `BIJUX_DAG_ADAPTERS_DIR` is set, bijux-dag discovers executables in that directory.
Each adapter binary must implement:

- `adapter info --json` returning:
```
{
  "id": "string",
  "version": "string",
  "required_effects": {"filesystem": true, "env": false, "network": false, "clock": false},
  "supported_kinds": ["kind-name"]
}
```
- `adapter execute --node-spec <json> --workdir <dir> --outdir <dir>`

Node traces include the adapter id/version plus a SHA256 of the adapter binary.
