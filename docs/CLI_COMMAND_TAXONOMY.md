# DAG CLI command taxonomy

## User-facing product commands

- `init`
- `validate`
- `canonicalize`
- `lint`
- `fingerprint`
- `run`
- `replay`
- `diff`
- `explain`
- `node`
- `status`
- `verify`
- `cache`
- `adapters`
- `export`
- `import`
- `version`

## Debug and diagnostics commands

- `doctor`

## Migration commands

- `migrate dag`
- `migrate run`

## Placement decisions

- `migrate` remains in the product CLI because migration is a user data-lifecycle operation.
- `doctor` remains in the product CLI because it is an operator diagnostics surface.
- `compat` is removed from product CLI; compatibility suite execution belongs to repository control-plane tooling.
