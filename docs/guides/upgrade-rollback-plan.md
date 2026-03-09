# Upgrade Rollback Plan

If an upgrade introduces regressions, roll back deterministically.

## Trigger Conditions

- Contract break in CLI output/envelopes
- Exit code regression affecting automation
- Install channel mismatch or unresolved path shadowing

## Rollback Steps

1. Stop promotion of the new version in deployment environments.
2. Reinstall last known-good version via the same channel used in production.
3. Verify with:
   `bijux version`
   `bijux cli paths`
   `bijux cli doctor`
4. Pin the rollback version in CI and deployment manifests.
5. Open a corrective release issue with impact and reproduction details.

## Exit Criteria

- Regression fixed and validated in CI
- New tag published with explicit migration note from rollback version

