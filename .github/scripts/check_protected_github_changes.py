#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path

PROTECTED_PATHS = {
    ".github/ISSUE_TEMPLATE/bug-report.yml",
    ".github/ISSUE_TEMPLATE/config.yml",
    ".github/ISSUE_TEMPLATE/feature-request.yml",
    ".github/PULL_REQUEST_TEMPLATE/default.md",
    ".github/PULL_REQUEST_TEMPLATE/release-change.md",
    ".github/automation-identity.md",
    ".github/required-status-checks.md",
    ".github/rulesets/main-branch-protection.json",
    ".github/scripts/check_pinned_actions.py",
    ".github/scripts/check_protected_github_changes.py",
    ".github/scripts/wait_for_ci.py",
    "shared/bijux-gh/workflows/build-release-artifacts.yml",
    "shared/bijux-gh/workflows/deploy-docs.yml",
    "shared/bijux-gh/workflows/release-artifacts.yml",
    "shared/bijux-gh/workflows/release-crates.yml",
    "shared/bijux-gh/workflows/release-ghcr.yml",
    "shared/bijux-gh/workflows/release-github.yml",
    "shared/bijux-gh/workflows/release-pypi.yml",
    "shared/bijux-gh/workflows/reusable-ci-python-packages.yml",
    "shared/bijux-gh/workflows/reusable-verify-python-packages.yml",
    "shared/bijux-gh/workflows/reusable-ci-rust-stack.yml",
    "shared/bijux-gh/workflows/github-policy.yml",
    ".github/workflows/bijux-std.yml",
    ".github/workflows/build-release-artifacts.yml",
    ".github/workflows/deploy-docs.yml",
    ".github/workflows/release-artifacts.yml",
    ".github/workflows/release-github.yml",
    ".github/workflows/reusable-ci-python-packages.yml",
    ".github/workflows/reusable-verify-python-packages.yml",
    ".github/workflows/reusable-ci-rust-stack.yml",
    ".github/workflows/github-policy.yml",
    ".github/bijux-std-shared.sha256",
    ".github/release.env",
    ".github/dependabot.yml",
    ".github/workflows/ci.yml",
    ".github/workflows/verify.yml",
    ".github/standards/bijux-std.sha",
}

ALLOWED_CONTROL_PATHS = {
    ".github/standards/repo-config.manifest.json",
    ".github/scripts/build_repo_manifest.py",
    ".github/scripts/render_repo_configs.py",
    ".github/scripts/sync_github_standards.py",
    ".github/bijux-std-shared.sha256",
}


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate protected .github changes")
    parser.add_argument("--changed-file", action="append", default=[], help="Changed file path (repeatable)")
    parser.add_argument("--changed-file-list", help="Path to newline-delimited changed files")
    args = parser.parse_args()

    changed = set(args.changed_file)
    if args.changed_file_list:
        changed.update(
            line.strip()
            for line in Path(args.changed_file_list).read_text(encoding="utf-8").splitlines()
            if line.strip()
        )

    protected_changed = sorted(path for path in changed if path in PROTECTED_PATHS)
    if not protected_changed:
        return 0

    controls_changed = sorted(path for path in changed if path in ALLOWED_CONTROL_PATHS)
    if controls_changed:
        return 0

    print("Protected .github files changed without approved generator/sync controls:")
    for path in protected_changed:
        print(f"  - {path}")
    print("Required: include at least one control path change in the same PR:")
    for path in sorted(ALLOWED_CONTROL_PATHS):
        print(f"  - {path}")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
