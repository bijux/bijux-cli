# Extensibility Contract

## Scope

Defines stable extension boundaries and explicitly rejects generic plugin claims
outside implemented surfaces.

## Implemented Extension Points

Current extension points:

- `task_adapter` (stable)
- `executor_backend` (experimental)

Internal hooks (not public plugin API):

- `validation_hook`

## Non-implemented Claims

No generic arbitrary plugin system is claimed.
Only documented extension points are supported.

## Extension Descriptor

External extension descriptor contract requires:

- plugin name
- plugin version
- boundary kind
- contract version (v-prefixed)
- declared capabilities
- trust model

Schema file:

- `configs/schema/extension_descriptor.schema.json`

## Versioning and Compatibility

- Extension interfaces are versioned by contract version (`vX.Y`).
- Unknown contract versions are rejected as compatibility issues.
- Required capabilities are validated before activation.

## Trust and Security Model

- Extensions must declare trust model.
- Signature/allowlist policy is validated by conformance checks.
- Extension failure must remain isolated from engine integrity.

## Lifecycle

- register
- validate
- execute
- deprecate
- remove

## Internal Hook Promotion

Internal hook promotion to public extension API requires:

- contract doc
- versioning policy
- negative tests
- failure isolation evidence

Checklist document:

- `docs/reference/INTERNAL_HOOK_PROMOTION_CHECKLIST.md`

## Verifying Surfaces

- `crates/bijux-dag-runtime/src/plugin_ecosystem.rs`
- `crates/bijux-dag-runtime/tests/plugin_ecosystem_contracts.rs`
- `bijux-dev-dag extension-report`
- `bijux-dev-dag repo` suite `extensibility-contract`
