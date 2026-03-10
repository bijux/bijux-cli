# Release Checklist

1. Confirm all CI jobs are green for the release commit.
2. Confirm package metadata consistency:
   `python3.11 scripts/check-package-metadata.py`
3. Confirm install channel verification:
   `bash scripts/verify-install-channels.sh`
4. Confirm docs build succeeds.
5. Review parity artifacts before release claims:
   - `artifacts/parity/command_parity_matrix.json`
   - `artifacts/parity/command_parity_summary.txt`
   - `artifacts/parity/parity_regression_summary.txt`
6. Create semantic tag `vX.Y.Z` on the release commit.
7. Verify PyPI publish status for `bijux-cli`.
8. Verify GitHub release assets include:
   checksum file, artifact manifest, release tarball, dist files.
9. Verify `bijux version` and `bijux cli doctor` on a clean environment.
10. Announce release with changelog link.
