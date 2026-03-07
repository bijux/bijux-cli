# Release Review Checklist

## Scope
Checklist for release approvers.

## Checklist
1. Public API surface review complete.
2. Run directory compatibility review complete.
3. Import/export format compatibility review complete.
4. Compatibility matrix generated and reviewed.
5. Benchmark regression report reviewed.
6. Resource profile regression report reviewed.
7. Known limitations section updated.
8. Reproducibility check report attached.
9. Post-release verification suite passed.
10. Mission and README drift review complete against `docs/spec/MISSION_STATEMENT.md`.

## Related tests
- `bijux-dev-dag release post-release-verify`

## Versioning and change policy
Checklist changes must remain aligned with `docs/spec/RELEASE_POLICY.md`.
