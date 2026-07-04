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
and command discovery. Hidden experimental routes and hidden simulation or
maintainer namespaces remain available by explicit path for repository-owned
workflows, but they are not part of the stable public operator contract.

## Deliberate boundaries

This crate stays thin. It does not own:

- graph semantics,
- runtime execution logic,
- artifact persistence rules.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
