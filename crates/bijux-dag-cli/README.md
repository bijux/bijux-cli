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
