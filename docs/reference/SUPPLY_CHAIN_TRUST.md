# Supply-chain trust, provenance, and compliance evidence

## Scope

This contract defines provenance and trust evidence requirements for scheduler, worker, CLI, plugin, run, and artifact outputs.

## Provenance records

- Binary provenance is captured for scheduler, worker, CLI, and plugin executables.
- Build identity is recorded using version, build identifier, source revision, and build timestamp.
- Plugin provenance includes source, trust tier, and approval status.

## Attestation and evidence

- Run attestations bind DAG snapshot, plan fingerprint, policy bundle version, binaries, plugins, and environment attestation.
- Signed artifact manifests are required for high-trust release workflows.
- Compliance evidence bundles collect run attestations plus signed manifests into immutable exports.

## Promotion and trust labels

Artifact trust labels:
- `unverified`
- `verified`
- `attested`
- `approved`

Promotion policy can require both:
- allowed trust labels
- provenance completeness checks

## Replay trust behavior

Replay checks are provenance-aware. They emit warnings when trust inputs differ from baseline, including:
- policy bundle version
- trust domain
- binary provenance
- plugin provenance

## Attestation compatibility

Attestation format changes are classified as:
- `compatible`
- `compatible-with-upgrade`
- `migration-required`
- `incompatible`

Unknown format transitions are treated as incompatible.

## CLI verification gate

`bijux-dev-dag` includes `attestation_verify`.

Usage:

```bash
cargo run -p bijux-dev-dag --bin attestation_verify -- attestation-input.json
```

The command exits non-zero when required provenance evidence is missing.

## Maturity model

- Local development: provenance optional.
- Shared development: binary provenance required.
- Staging: provenance and attestation verification required.
- Production: signed artifacts and trust-label-based promotion required.
- High assurance: full attestation, signed artifacts, and explicit approval gates required.
