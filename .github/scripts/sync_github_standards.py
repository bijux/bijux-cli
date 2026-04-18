#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
STD_REPO = ROOT / "bijux-std"
PIN_PATH = ".github/standards/bijux-std.sha"

DEFAULT_REPOS = [
    "bijux-atlas",
    "bijux-canon",
    "bijux-core",
    "bijux-masterclass",
    "bijux-pollenomics",
    "bijux-proteomics",
    "bijux.github.io",
]

SHARED_FILE_MAPPINGS: list[tuple[str, str]] = [
    (".github/ISSUE_TEMPLATE/bug-report.yml", ".github/ISSUE_TEMPLATE/bug-report.yml"),
    (".github/ISSUE_TEMPLATE/config.yml", ".github/ISSUE_TEMPLATE/config.yml"),
    (".github/ISSUE_TEMPLATE/feature-request.yml", ".github/ISSUE_TEMPLATE/feature-request.yml"),
    (".github/PULL_REQUEST_TEMPLATE/default.md", ".github/PULL_REQUEST_TEMPLATE/default.md"),
    (".github/PULL_REQUEST_TEMPLATE/release-change.md", ".github/PULL_REQUEST_TEMPLATE/release-change.md"),
    (".github/automation-identity.md", ".github/automation-identity.md"),
    (".github/required-status-checks.md", ".github/required-status-checks.md"),
    (".github/rulesets/main-branch-protection.json", ".github/rulesets/main-branch-protection.json"),
    (".github/scripts/build_repo_manifest.py", ".github/scripts/build_repo_manifest.py"),
    (".github/scripts/check_pinned_actions.py", ".github/scripts/check_pinned_actions.py"),
    (".github/scripts/check_protected_github_changes.py", ".github/scripts/check_protected_github_changes.py"),
    (".github/scripts/render_repo_configs.py", ".github/scripts/render_repo_configs.py"),
    (".github/scripts/sync_github_standards.py", ".github/scripts/sync_github_standards.py"),
    (".github/scripts/wait_for_ci.py", ".github/scripts/wait_for_ci.py"),
    (".github/standards/repo-config.manifest.json", ".github/standards/repo-config.manifest.json"),
    (".github/workflows/bijux-std.yml", ".github/workflows/bijux-std.yml"),
    ("shared/bijux-gh/workflows/build-release-artifacts.yml", "shared/bijux-gh/workflows/build-release-artifacts.yml"),
    ("shared/bijux-gh/workflows/deploy-docs.yml", "shared/bijux-gh/workflows/deploy-docs.yml"),
    ("shared/bijux-gh/workflows/release-artifacts.yml", "shared/bijux-gh/workflows/release-artifacts.yml"),
    ("shared/bijux-gh/workflows/release-crates.yml", "shared/bijux-gh/workflows/release-crates.yml"),
    ("shared/bijux-gh/workflows/release-ghcr.yml", "shared/bijux-gh/workflows/release-ghcr.yml"),
    ("shared/bijux-gh/workflows/release-github.yml", "shared/bijux-gh/workflows/release-github.yml"),
    ("shared/bijux-gh/workflows/release-pypi.yml", "shared/bijux-gh/workflows/release-pypi.yml"),
    ("shared/bijux-gh/workflows/reusable-ci-python-packages.yml", "shared/bijux-gh/workflows/reusable-ci-python-packages.yml"),
    ("shared/bijux-gh/workflows/reusable-verify-python-packages.yml", "shared/bijux-gh/workflows/reusable-verify-python-packages.yml"),
    ("shared/bijux-gh/workflows/reusable-ci-rust-stack.yml", "shared/bijux-gh/workflows/reusable-ci-rust-stack.yml"),
    ("shared/bijux-gh/workflows/github-policy.yml", "shared/bijux-gh/workflows/github-policy.yml"),
    ("shared/bijux-gh/workflows/build-release-artifacts.yml", ".github/workflows/build-release-artifacts.yml"),
    ("shared/bijux-gh/workflows/deploy-docs.yml", ".github/workflows/deploy-docs.yml"),
    ("shared/bijux-gh/workflows/release-artifacts.yml", ".github/workflows/release-artifacts.yml"),
    ("shared/bijux-gh/workflows/release-crates.yml", ".github/workflows/release-crates.yml"),
    ("shared/bijux-gh/workflows/release-ghcr.yml", ".github/workflows/release-ghcr.yml"),
    ("shared/bijux-gh/workflows/release-github.yml", ".github/workflows/release-github.yml"),
    ("shared/bijux-gh/workflows/release-pypi.yml", ".github/workflows/release-pypi.yml"),
    ("shared/bijux-gh/workflows/reusable-ci-python-packages.yml", ".github/workflows/reusable-ci-python-packages.yml"),
    ("shared/bijux-gh/workflows/reusable-verify-python-packages.yml", ".github/workflows/reusable-verify-python-packages.yml"),
    ("shared/bijux-gh/workflows/reusable-ci-rust-stack.yml", ".github/workflows/reusable-ci-rust-stack.yml"),
    ("shared/bijux-gh/workflows/github-policy.yml", ".github/workflows/github-policy.yml"),
    (".github/bijux-std-shared.sha256", ".github/bijux-std-shared.sha256"),
]


def run(cmd: list[str], cwd: Path | None = None) -> str:
    result = subprocess.run(cmd, cwd=cwd, check=True, text=True, capture_output=True)
    return result.stdout.strip()


def copy_shared_files(target_repo: str) -> None:
    for source_relative, destination_relative in SHARED_FILE_MAPPINGS:
        source = STD_REPO / source_relative
        destination = ROOT / target_repo / destination_relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(source.read_bytes())


def write_std_pin(repo_name: str, std_sha: str) -> None:
    pin_file = ROOT / repo_name / PIN_PATH
    pin_file.parent.mkdir(parents=True, exist_ok=True)
    pin_file.write_text(f"{std_sha}\n", encoding="utf-8")


def has_changes(repo_name: str) -> bool:
    status = run(["git", "status", "--short"], cwd=ROOT / repo_name)
    return bool(status)


def ensure_branch(repo_dir: Path, branch_name: str) -> None:
    existing = run(["git", "branch", "--list", branch_name], cwd=repo_dir)
    if existing:
        run(["git", "checkout", branch_name], cwd=repo_dir)
    else:
        run(["git", "checkout", "-b", branch_name], cwd=repo_dir)


def create_pr(repo_dir: Path, args: argparse.Namespace, branch_name: str) -> dict | None:
    if not args.open_pr:
        return None

    run(["git", "push", "-u", "origin", branch_name], cwd=repo_dir)
    pr_json = run(
        [
            "gh",
            "pr",
            "create",
            "--base",
            args.base_branch,
            "--head",
            branch_name,
            "--title",
            args.pr_title,
            "--body",
            args.pr_body,
            "--json",
            "number,url",
        ],
        cwd=repo_dir,
    )
    return json.loads(pr_json)


def wait_for_merge(repo_dir: Path, pr_number: int, timeout_seconds: int, interval_seconds: int) -> dict:
    deadline = time.time() + timeout_seconds
    while True:
        info = json.loads(
            run(
                [
                    "gh",
                    "pr",
                    "view",
                    str(pr_number),
                    "--json",
                    "number,url,state,mergeStateStatus,isDraft",
                ],
                cwd=repo_dir,
            )
        )
        if info.get("state") == "MERGED":
            return {"status": "merged", "details": info}
        if info.get("state") == "CLOSED":
            return {"status": "closed", "details": info}

        if time.time() >= deadline:
            return {"status": "timeout", "details": info}

        time.sleep(interval_seconds)


def main() -> None:
    parser = argparse.ArgumentParser(description="Sync shared .github standards into consumer repositories")
    parser.add_argument("--repo", action="append", default=[], help="Repository name (repeatable)")
    parser.add_argument("--create-branch", action="store_true", help="Create per-repo branch before commit")
    parser.add_argument("--open-pr", action="store_true", help="Push and open PR for each changed repository")
    parser.add_argument("--track-merge-status", action="store_true", help="Poll opened PRs until merged/closed/timeout")
    parser.add_argument("--merge-timeout-seconds", type=int, default=3600, help="Maximum polling duration per PR")
    parser.add_argument("--merge-poll-interval-seconds", type=int, default=60, help="Polling interval")
    parser.add_argument("--advance-std-sha", action="store_true", help="Write current bijux-std commit SHA pin into each target repo")
    parser.add_argument("--base-branch", default="main", help="PR base branch")
    parser.add_argument("--branch-prefix", default="chore/github-standards-sync", help="Branch prefix")
    parser.add_argument("--commit-message", default="chore(github): sync shared standards and generated config", help="Commit message")
    parser.add_argument("--pr-title", default="chore(github): sync shared standards and generated config", help="PR title")
    parser.add_argument("--pr-body", default="Synchronize shared .github templates from bijux-std and regenerate repository-local config files.", help="PR body")
    args = parser.parse_args()

    repos = args.repo or DEFAULT_REPOS
    std_sha = run(["git", "rev-parse", "HEAD"], cwd=STD_REPO)

    render_script = STD_REPO / ".github/scripts/render_repo_configs.py"
    subprocess.run(["python3", str(render_script), "--repo", "bijux-std"], check=True)

    pr_records: list[tuple[str, int, str]] = []
    changed_repos: list[str] = []

    for repo in repos:
        repo_dir = ROOT / repo
        copy_shared_files(repo)
        subprocess.run(["python3", str(render_script), "--repo", repo], check=True)

        if args.advance_std_sha:
            write_std_pin(repo, std_sha)

        subprocess.run(["sha256sum", "--check", ".github/bijux-std-shared.sha256"], cwd=repo_dir, check=True)

        if not has_changes(repo):
            continue

        changed_repos.append(repo)
        branch_name = f"{args.branch_prefix}/{repo}"
        if args.create_branch:
            ensure_branch(repo_dir, branch_name)

        run(["git", "add", ".github"], cwd=repo_dir)
        run(["git", "commit", "-m", args.commit_message], cwd=repo_dir)

        pr_info = create_pr(repo_dir, args, branch_name)
        if pr_info:
            pr_records.append((repo, pr_info["number"], pr_info["url"]))

    print("changed_repositories:")
    for repo in changed_repos:
        print(f"  - {repo}")

    if args.track_merge_status and pr_records:
        print("merge_status:")
        for repo, pr_number, pr_url in pr_records:
            result = wait_for_merge(
                ROOT / repo,
                pr_number,
                timeout_seconds=args.merge_timeout_seconds,
                interval_seconds=args.merge_poll_interval_seconds,
            )
            print(f"  - {repo}: {result['status']} ({pr_url})")


if __name__ == "__main__":
    main()
