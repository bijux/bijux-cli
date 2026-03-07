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

Readiness and evidence linkage are tracked in [renovation burndown report](./reports/foundation/renovation_burndown_report.md).
