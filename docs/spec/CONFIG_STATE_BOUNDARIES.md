# Config, Policy, Runtime State, and Artifacts

## Scope
Defines boundaries between configuration classes.

## Boundaries
- `config`: user-supplied static settings resolved before run start.
- `policy`: behavioral constraints enforced during planning/execution.
- `runtime state`: ephemeral in-memory execution status.
- `artifacts`: persisted run outputs, traces, and manifests.

## Invariants
- Config and policy are inputs to runtime behavior and must be representable in machine-readable forms.
- Runtime state is not a config source.
- Artifacts record outcomes, not unresolved policy/config intent.

## Related tests
- `crates/bijux-dag-app/tests/config_precedence_contract.rs`
- `crates/bijux-dag-app/tests/cache_invalidation_config_contract.rs`

## Versioning and change policy
Boundary changes require contract updates in CONFIG/POLICY/RUN_DIR docs and corresponding test updates.
