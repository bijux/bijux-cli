# Upgrade, migration, and compatibility governance

## Compatibility policy surface

Compatibility classifications are tracked for:
- DAG specifications
- run manifests
- artifact manifests
- API contracts
- plugin interfaces
- scheduler durable state

Classes:
- fully-compatible
- replay-compatible
- read-only-compatible
- migration-required
- breaking

## Migration tooling contracts

Migration plans define:
- source version and target version
- deterministic transformation steps
- post-migration validation checks

Supported migrations include DAG spec, run manifest, artifact manifest, lineage index, and durable scheduler/registry state.

## Plugin version negotiation and deprecation windows

Plugin interfaces include explicit support windows and deprecation deadlines. Runtime negotiation must fail fast outside the accepted range.

## Downgrade risk and feature lifecycle

Downgrade risk reports identify incompatible surfaces and whether rollback is blocked.

Feature lifecycle states are strict:
- experimental
- preview
- stable
- deprecated
- removed

Deprecation diagnostics are required in CLI, API, and release evidence.

## Upgrade simulation and rollout

`migration_simulate` provides upgrade impact estimates before rollout.

```bash
cargo run -p bijux-dev-dag --bin migration_simulate -- migration-simulate-input.json
```

Blue/green and canary rollout policy requires:
- state compatibility checks before HA or sharding upgrades
- canary verification evidence
- explicit rollback strategy

## Release and acceptance governance

Required release gates:
- mandatory compatibility acceptance suites pass
- no unreviewed breaking changes

## Long-term support and dashboard

LTS retention windows are defined for core specs, API, and official plugin interfaces.

Compatibility dashboard summarizes:
- policy classes by surface
- rule inventory
- feature state distribution
- required suite coverage
- downgrade blocking risk
