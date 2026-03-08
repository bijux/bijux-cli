# Compatibility and upgrade governance

Audience: maintainers and advanced users.
Owner: architecture, runtime, and release owners.
Status: stable.

## Compatibility surface

- Specification versions (current major contract line: `bijux-dag/v0.1`).
- Canonical JSON and fingerprint contracts.
- CLI contract surfaces and parser behavior.

Unknown spec values must fail strict parsing for invalid compatibility surfaces.
Canonical JSON and fingerprint contracts remain stable for unchanged graph semantics.

## Runtime and manifest compatibility

- Replay and diff compare graph fingerprints and stable run-manifest shapes.
- Deterministic output ordering and schema stability are required for equivalent inputs.
- Diff contracts should remain deterministic for repeated identical runs.

## CLI compatibility classes

Contracted CLI commands include:

- `dag validate`, `run`, `replay`, `diff`, `status`, `cache`, `adapters`
- `dag hash run`, `dag hash artifact`, `dag fsck`
- JSON envelope shape
- Stable parser status-code classes for validation/runtime failures

## Upgrade and compatibility policy

Compatibility classifications are tracked for:

- DAG specifications
- run manifests
- artifact manifests
- API contracts
- plugin interfaces
- scheduler durable state

Classifications used in governance:

- fully-compatible
- replay-compatible
- read-only-compatible
- migration-required
- deprecated (remove per deprecation windows)

## Upgrade process

Migration plans include:

- source and target versions
- deterministic transformation steps
- post-migration validation checks

Supported migrations include DAG specs, run manifests, artifact manifests, lineage indexes, and durable scheduler/registry state.

## Compatibility windows

For spec family `v0.1`, backward compatibility for canonical parsing, validation diagnostics, and run-artifact shape is maintained across `0.1.x` releases.

## Deprecation and lifecycle

- Breaking surface changes require explicit deprecation notice.
- Experimental features must not be interpreted as stable compatibility commitments.
- Plugin and interface compatibility deadlines are published in release governance materials.

Canonical compatibility details belong in:

- [Migration policy](./spec/MIGRATION_POLICY.md)
- [Schema compatibility policy](./spec/SCHEMA_COMPATIBILITY_POLICY.md)
- [CLI compatibility contract](./spec/CLI_BACKWARD_COMPATIBILITY.md)
