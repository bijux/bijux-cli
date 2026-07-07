# bijux-dag-cli

`bijux-dag-cli` installs the `bijux-dag` executable. It is the publishable,
user-facing command package for the DAG product.

## What this crate provides

- The `bijux-dag` binary entrypoint.
- Thin startup wiring, process initialization, and exit mapping.
- Delegation into `bijux-dag-app` for actual command behavior.

Install it from crates.io when you want the standalone DAG command surface:

```bash
cargo install bijux-dag-cli
bijux-dag --help
```

The supported release boundary is the visible `bijux-dag --help` surface.
That visible root surface stays intentionally concise for `v0.4.0`: validate,
plan, run, replay, inspect-oriented routes, cache operations, doctor, version,
and command discovery. Hidden experimental routes remain available by explicit
path for repository-owned workflows. Hidden simulation and maintainer
namespaces require explicit opt-in through `BIJUX_DAG_ENABLE_SIMULATED=1` or
`BIJUX_DAG_ENABLE_INTERNAL=1`, and they are not part of the stable public
operator contract.

## Deliberate boundaries

This crate stays thin. It does not own:

- graph semantics,
- runtime execution logic,
- artifact persistence rules.

## What the binary owns

- process startup and CLI initialization
- handoff into `bijux-dag-app`
- exit-code mapping and top-level process behavior
- shell completion generation for the installed executable

If the question is about route behavior rather than process startup, the owning
crate is usually `bijux-dag-app`.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
