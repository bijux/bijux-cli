#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path


WORKSPACE_PACKAGES = {"bijux-cli", "bijux-cli-python", "bijux-dev-cli"}
IGNORE_NAMES = {
    ".git",
    ".DS_Store",
    ".direnv",
    "artifacts",
    "build",
    "dist",
    "htmlcov",
    ".hypothesis",
    ".idea",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "__pycache__",
    "target",
    "venv",
    ".venv",
}
IGNORE_PREFIXES = (".coverage",)
IGNORE_SUFFIXES = (".egg-info", ".dSYM")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a temporary release tree with workspace package versions stamped to a tag."
    )
    parser.add_argument("--workspace-root", required=True, help="Path to the source workspace root.")
    parser.add_argument("--output-dir", required=True, help="Path to the prepared release tree.")
    parser.add_argument("--version", required=True, help="Release version without a leading v.")
    return parser.parse_args()


def ensure_clean_output_dir(path: Path) -> None:
    if path.exists() and any(path.iterdir()):
        raise SystemExit(f"output directory must be empty: {path}")
    path.mkdir(parents=True, exist_ok=True)


def should_ignore(name: str) -> bool:
    return (
        name in IGNORE_NAMES
        or name.startswith(IGNORE_PREFIXES)
        or name.endswith(IGNORE_SUFFIXES)
    )


def ignore_entries(_dir: str, names: list[str]) -> set[str]:
    return {name for name in names if should_ignore(name)}


def copy_workspace(workspace_root: Path, output_dir: Path) -> None:
    for entry in workspace_root.iterdir():
        if should_ignore(entry.name):
            continue
        destination = output_dir / entry.name
        if entry.is_dir():
            shutil.copytree(entry, destination, ignore=ignore_entries)
        else:
            shutil.copy2(entry, destination)


def rewrite_workspace_version(path: Path, release_version: str) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    out: list[str] = []
    in_workspace_package = False
    replaced = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_workspace_package = stripped == "[workspace.package]"
        if in_workspace_package and line.lstrip().startswith("version = "):
            indent = line[: len(line) - len(line.lstrip())]
            out.append(f'{indent}version = "{release_version}"')
            replaced = True
            continue
        out.append(line)
    if not replaced:
        raise SystemExit(f"failed to rewrite [workspace.package] version in {path}")
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def rewrite_lockfile_versions(path: Path, release_version: str) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    out: list[str] = []
    current_package: str | None = None
    in_package_block = False
    replaced = 0

    for line in lines:
        stripped = line.strip()
        if stripped == "[[package]]":
            in_package_block = True
            current_package = None
            out.append(line)
            continue
        if stripped.startswith("[") and stripped != "[[package]]":
            in_package_block = False
            current_package = None
        if in_package_block and stripped.startswith('name = "'):
            current_package = stripped.removeprefix('name = "').removesuffix('"')
            out.append(line)
            continue
        if in_package_block and current_package in WORKSPACE_PACKAGES and stripped.startswith("version = "):
            indent = line[: len(line) - len(line.lstrip())]
            out.append(f'{indent}version = "{release_version}"')
            replaced += 1
            continue
        out.append(line)

    if replaced != len(WORKSPACE_PACKAGES):
        raise SystemExit(
            f"expected to rewrite {len(WORKSPACE_PACKAGES)} workspace package versions in {path}, rewrote {replaced}"
        )
    path.write_text("\n".join(out) + "\n", encoding="utf-8")


def parse_release_version(release_version: str) -> tuple[int, int, int]:
    parts = release_version.split(".")
    if len(parts) != 3 or any(not part.isdigit() for part in parts):
        raise SystemExit(f"release version must be x.y.z semver without prerelease/build metadata: {release_version}")
    return int(parts[0]), int(parts[1]), int(parts[2])


def next_supported_host_boundary(release_version: str) -> str:
    major, minor, _patch = parse_release_version(release_version)
    if major == 0:
        return f"0.{minor + 1}.0"
    return f"{major + 1}.0.0"


def rewrite_template_compatibility_defaults(path: Path, release_version: str) -> None:
    payload = json.loads(path.read_text(encoding="utf-8"))
    payload["cli_min"] = release_version
    payload["cli_max"] = next_supported_host_boundary(release_version)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    args = parse_args()
    workspace_root = Path(args.workspace_root).resolve()
    output_dir = Path(args.output_dir).resolve()
    release_version = args.version.strip().removeprefix("v")
    if not release_version:
        raise SystemExit("release version must not be empty")

    ensure_clean_output_dir(output_dir)
    copy_workspace(workspace_root, output_dir)
    rewrite_workspace_version(output_dir / "Cargo.toml", release_version)
    rewrite_lockfile_versions(output_dir / "Cargo.lock", release_version)
    rewrite_template_compatibility_defaults(
        output_dir / "templates/plugins-py/cookiecutter.json",
        release_version,
    )
    rewrite_template_compatibility_defaults(
        output_dir / "templates/plugins-rs/cookiecutter.json",
        release_version,
    )
    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
