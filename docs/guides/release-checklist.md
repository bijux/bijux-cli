# Release Checklist

1. Confirm all CI jobs are green for the release commit.
2. Confirm package metadata consistency:
   `python3.11 scripts/check-package-metadata.py`
3. Confirm install channel verification:
   `bash scripts/verify-install-channels.sh`
4. Confirm docs build succeeds.
5. Create semantic tag `vX.Y.Z` on the release commit.
6. Verify PyPI publish status for `bijux-cli`.
7. Verify GitHub release assets include:
   checksum file, artifact manifest, release tarball, dist files.
8. Verify `bijux version` and `bijux cli doctor` on a clean environment.
9. Announce release with changelog link.

