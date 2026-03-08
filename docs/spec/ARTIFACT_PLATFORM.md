# Artifact platform contracts

This document defines artifact-platform contracts for multi-store storage, exchange, lineage operations, and verification governance.

## Store routing and replication

- `ArtifactStoreClass`: `HotCache`, `DurableLocal`, `RemoteObject`.
- `ArtifactStoreRoute` binds logical `ArtifactId` to an explicit store class and storage key.
- `ArtifactReplicationRule` and `ArtifactReplicationRecord` define deterministic replication and promotion evidence.

## Packing, compression, and chunking

- `ArtifactPackingProfile` defines policy surfaces for replay, archive, compliance, and handoff.
- `ArtifactCompressionPolicy` encodes deterministic compression requirements.
- `ArtifactChunkPolicy` and `ArtifactChunkDescriptor` define chunking without losing content identity observability.

## Provenance, verification, and trust hooks

- `ArtifactSigningHook` reserves manifest-signing integration points.
- `ArtifactProvenanceRecord` extends provenance with producer binary identity, adapter version, and environment class.
- `ArtifactVerificationReport` provides release and audit-friendly verification output.

## Retention and safe collection planning

- `ArtifactRetentionClass` supports operational/legal classes: transient, retained, release, audit.
- `ArtifactGarbageCollectionPlan` separates preserved artifacts from collectable artifacts while tying decisions to lineage snapshot identity.

## Import/export and sensitive data controls

- `ArtifactImportCompatibility` records compatibility checks across source/target spec versions and environments.
- `ArtifactExportProfile` covers handoff, backup, replication, and compliance evidence bundles.
- `ArtifactRedactionPolicy` defines log/metadata redaction controls.
- `ImmutableArtifactAnnotation` provides operator-added context without mutating content identity.

## Lineage query and replay assist

- `compact_lineage` builds a producer-oriented compact lineage index.
- `lineage_dependencies` and `lineage_dependents` support “what produced this” and “what depends on this”.
- `build_replay_assist` emits minimal upstream context for selected artifact replay.

## Store conformance

`run_store_conformance` defines a baseline backend contract:

- write succeeds
- read succeeds
- read content equals written content

Backends are expected to pass this baseline before being promoted for runtime support.
