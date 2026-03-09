# EXTENSIBILITY AND PLUGIN CONTRACTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ADOPTION_SURFACES.md
# Adoption Surfaces

## Official product surfaces
- CLI command tree (`dag ...`)
- run-directory artifacts
- export bundles
- config and schema files

## Crate stability levels
- internal:
  - `bijux-dag-runtime`
  - `bijux-dag-core`
  - `bijux-dag-artifacts`
- experimental:
  - public crate use outside the CLI binary
- supported:
  - CLI interface and documented JSON contracts

## External consumption policy
- Rust crates are internal-first unless explicitly documented as stable APIs.
- Quickstarts and installation docs must reference only supported surfaces.

## Machine-readable capability surface
- `dag capabilities --json` is the canonical summary of currently supported and simulated surfaces.

## SOURCE: docs/spec/EXTENSIBILITY_CONTRACT.md
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

- `crates/bijux-dag-runtime/src/extension_catalog.rs`
- `crates/bijux-dag-runtime/tests/extension_catalog_contracts.rs`
- `bijux-dev-dag extension-report`
- `bijux-dev-dag repo` suite `extensibility-contract`

## SOURCE: docs/spec/EXTENSION_CATALOG_CONTRACTS.md
# Extension catalog contracts

This document defines stable extension boundaries and governance contracts for adapter, executor, artifact-store, and observability plugins.

## Stable plugin boundaries

Supported boundaries:

- task adapter
- executor backend
- artifact store
- observability sink/exporter

Each plugin declares boundary kind, capabilities, policy requirements, and compatibility range.

## Metadata and version negotiation

`PluginMetadata` includes:

- name
- version
- boundary
- capabilities
- policy requirements
- compatibility range

Version negotiation uses explicit contract-version bounds.

## Loading strategy

- static linking is the default and supported path.
- dynamic loading is intentionally deferred until trust and isolation controls are mature.

## Registration and discovery

- `ExtensionRegistration` is recorded for all enabled plugins.
- discovery inventory lists active extensions by boundary/capability.
- registration records are expected in manifests and diagnostics for provenance.

## Trust, security, and isolation

Trust controls:

- optional signing requirement
- publisher allowlist
- allowed environment classes

Isolation controls:

- deny undeclared effects
- require deterministic policy mode
- enforce resource caps

## SDK examples

Adapter patterns:

- local process task adapter
- container task adapter
- remote service task adapter

Artifact store patterns:

- filesystem-backed store
- object storage-backed store

Observability exporter patterns:

- structured file sink
- OTLP-compatible sink

## Compatibility and lifecycle

Conformance suite checks:

- metadata completeness
- trust policy constraints
- isolation policy constraints
- contract version compatibility

Lifecycle states:

- develop
- register
- validate
- release
- deprecate
- remove

## Official vs community plugins

Official plugins require core-team review and security assessment. Community plugins remain supported through stable contract boundaries but are not distributed as official defaults.

## DSL and code generation extension points

- DSL extension points for custom node families remain compile-time validated.
- code generation hooks may emit schema bindings or task-contract bindings.

## Ecosystem scope and maturity

- Core: deterministic graph/runtime contracts and built-in adapters.
- Pluggable: adapters, executors, artifact stores, observability exporters.
- Intentionally unsupported: plugins that bypass policy gates or deterministic guarantees.

Readiness and evidence linkage are tracked in [renovation burndown report](./reports/foundation/RENOVATION_BURNDOWN_REPORT.md).
