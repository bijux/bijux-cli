# Release Checklist

1. Confirm all CI jobs are green for the release commit.
2. Confirm release evidence artifacts are generated:
   - `artifacts/status/release_evidence_bundle.json`
   - `artifacts/status/release_status_manifest.json`
   - `artifacts/status/release_truth_report.json`
3. Review intentionally different behaviors:
   - `artifacts/status/status_intentional_differences.json`
4. Review unresolved partial commands:
   - `artifacts/status/what_is_partial.json`
5. Review stale scripts still outside `dev cli`:
   - `artifacts/status/script_only_behaviors.json`
6. Review stale docs flagged by docs audit:
   - `artifacts/status/docs_audit.json`
7. Review weak tests flagged by test audit:
   - `artifacts/status/test_quality_audit.json`
8. Confirm package metadata consistency:
   `python3.11 scripts/check-package-metadata.py`
9. Confirm install channel verification:
   `bash scripts/verify-install-channels.sh`
10. Confirm docs build succeeds.
11. Review parity artifacts before release claims:
   - `artifacts/parity/command_parity_matrix.json`
   - `artifacts/parity/command_parity_summary.txt`
   - `artifacts/parity/parity_regression_summary.txt`
12. Create semantic tag `vX.Y.Z` on the release commit.
13. Verify PyPI publish status for `bijux-cli`.
14. Verify GitHub release assets include:
   checksum file, artifact manifest, release tarball, dist files.
15. Verify `bijux version` and `bijux cli doctor` on a clean environment.
16. Announce release with changelog link and link `artifacts/status/release_truth_report.txt`.
