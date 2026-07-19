# Artifact Contract Tests

This suite protects the durable evidence written for a DAG run. It tests
artifact identity, storage layout, manifests, lineage, retention, and
corruption handling independently from command rendering and runtime
scheduling.

## Coverage

- canonical artifact and run-manifest round trips
- content hashes, logical identity, lineage, and metadata normalization
- filesystem store behavior, atomic persistence, and retention
- malformed, missing, and corrupted artifact refusal
- public Rust API and conformance surfaces
- failure summaries and resource accounting stored with run evidence

Fixtures must describe a storage or compatibility case that cannot be expressed
more clearly with a temporary directory built by the test. Persisted fixture
formats are contracts: update the reader, writer, migration behavior, and
compatibility evidence together.

## Focused Runs

```bash
cargo nextest run -p bijux-dag-artifacts --test artifact_identity_and_lineage_contracts
cargo nextest run -p bijux-dag-artifacts --test run_manifest_roundtrip_and_retention_contracts
cargo nextest run -p bijux-dag-artifacts --test artifact_storage_resilience_contracts
```

A failed hash or round-trip assertion is not snapshot noise. Determine whether
the serialized contract changed intentionally, whether nondeterministic input
entered canonicalization, or whether the store violated atomicity. Keep
filesystem output under the test temporary directory or repository
`artifacts/`.
