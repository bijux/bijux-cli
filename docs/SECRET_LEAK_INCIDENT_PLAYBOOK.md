# Secret leak incident response playbook

This playbook covers suspected secret leakage through runs, artifacts, logs, or diagnostics.

## Immediate containment

1. identify affected run IDs, nodes, and channels (stdout/stderr/manifest/export).
2. revoke impacted credentials and secret versions.
3. quarantine affected runs and block artifact promotion.
4. enforce strict secure-execution mode for impacted tenant/environment.

## Eradication and cleanup

1. trigger secure teardown and workspace cleanup for affected workers.
2. remove leaked secret-bearing artifacts and regenerate sanitized outputs.
3. verify masking/redaction policy enforcement in logs and manifests.

## Recovery

1. rotate secrets and reissue bounded credentials.
2. replay affected runs with audited secure configuration.
3. verify readiness checks for secret source integrations.

## Post-incident evidence

- credential provenance records
- authentication and revocation events
- run snapshots and policy traces
- lineage links to leaked artifacts

## Preventive controls

- leak conformance fixtures must pass in CI
- strict secret delivery policy in hardened environments
- regular review of tenant secret scopes and plugin allowlists
