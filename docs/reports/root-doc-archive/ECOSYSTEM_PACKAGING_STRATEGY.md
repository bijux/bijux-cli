# Ecosystem packaging and productization strategy

## Official packaging modes

- standalone binary
- container image
- reference deployment bundles

All official distributions require signing and verification.

## Deployment profiles

Reference deployment profiles cover:
- local development
- single-node production
- HA control plane
- federated platform

## Compatibility and support

Each release publishes a compatibility matrix for:
- backends
- stores
- auth providers
- plugins

Support policy distinguishes official integrations from community integrations.

## Upgrade and transparency

Upgrade bundles must include migration checks, compatibility reports, and rollback guidance.

Release transparency includes benchmark and capability summaries.

## Installation diagnostics and conformance

Install diagnostics validate deployment readiness before workload onboarding.

Conformance tests are required for each official packaging mode.
