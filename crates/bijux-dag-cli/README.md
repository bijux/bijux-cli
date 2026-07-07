# bijux-dag-cli

`bijux-dag-cli` installs the `bijux-dag` executable. It is the publishable,
user-facing command package for the DAG product.

Install it from crates.io when you want the standalone DAG command surface:

```bash
cargo install bijux-dag-cli
bijux-dag --help
```

## Release Status

- public crate on the `v0.4.0` DAG release line
- installs the stable operator-facing `bijux-dag` binary
- does not promote experimental, simulated, or internal namespaces into the
  default public contract

## Stable Operator Boundary

The supported release boundary is the visible `bijux-dag --help` surface:
`validate`, `plan`, `run`, `replay`, `runs`, `artifact`, `artifact-inspect`,
`diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, and
`completions`.

Experimental routes remain available by explicit path for repository-owned
workflows. Simulated and maintainer namespaces require explicit opt-in through
`BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1`.

## What This Crate Owns

- the `bijux-dag` binary entrypoint
- thin startup wiring, process initialization, and exit mapping
- delegation into `bijux-dag-app` for actual command behavior
- shell completion generation for the installed executable

## What It Does Not Own

- graph semantics
- runtime execution logic
- artifact persistence rules

If the question is about route behavior rather than process startup, the owning
crate is usually `bijux-dag-app`.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
