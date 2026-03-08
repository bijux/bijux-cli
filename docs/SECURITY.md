# Security and policy

Audience: operators and maintainers.
Owner: security owners.
Status: stable.

## Policy gates

Bijux provides policy flags to enforce organization-level execution rules before node execution:

- `--deny-network` blocks nodes that declare `network` effects.
- `--deny-env` blocks nodes that declare `env` effects.
- `--deny-clock` blocks nodes that declare `clock` effects.

Policy violations fail as policy errors.

## Runtime and command hardening

- Commands enforce explicit effects and deny-list policy flags.
- Network, environment, and clock effects are rejected at runtime when explicitly denied.
- Shell execution remains restricted by declared effects in each node.
- `bijux` is the only supported public CLI entrypoint; use `bijux dag ...` for DAG operations.

## Secret-handling and secure authoring

Use `SecretReference` contracts instead of plain secrets in parameters or environment values.
Run secret resolution at runtime unless compile-time resolution is explicitly allowed.

Required controls:

- Mask secrets in logs, diagnostics, manifests, and exports.
- Prefer file-mount or backend-native secret delivery in hardened environments.
- Pin secret versions for backfills and replay-critical workflows.
- Apply strict secret-taint handling for logs, diagnostics, and artifacts.

## Secret-taint governance and anti-patterns

Avoid:

- passing secrets as process arguments
- writing secrets to stdout/stderr
- embedding secrets in manifests
- storing secret-derived artifacts in general stores without retention controls
- bypassing authentication in non-local environments

## Security supply chain and trust evidence

- Supply-chain checks and dependency controls are run through repository control-plane checks.
- Release and safety gates run via `bijux-dev-dag` evidence evidence suites.

## Incident handling

- `docs/SECRET_LEAK_INCIDENT_PLAYBOOK.md` documents operational response for detected leaks.

## Links to contracts

Normative security and policy contracts live in:

- [Policy contract](./spec/POLICY_CONTRACT.md)
- [Authentication and identity trust](./AUTH_IDENTITY_TRUST.md)
- [Supply-chain trust overview](./SUPPLY_CHAIN_TRUST.md)
- [Authentication and identity contract](./AUTH_IDENTITY_TRUST.md)
