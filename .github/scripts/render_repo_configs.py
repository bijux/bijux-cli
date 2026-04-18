#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = ROOT / "bijux-std/.github/standards/repo-config.manifest.json"


def yaml_scalar(value: Any) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if value is None:
        return "null"

    text = str(value)
    if re.fullmatch(r"[A-Za-z0-9_./:-]+", text):
        return text
    escaped = text.replace('\\', '\\\\').replace('"', '\\"')
    return f'"{escaped}"'


def dump_yaml(obj: Any, indent: int = 0) -> list[str]:
    pad = " " * indent
    lines: list[str] = []

    if isinstance(obj, dict):
        for key, value in obj.items():
            if isinstance(value, (dict, list)):
                lines.append(f"{pad}{key}:")
                lines.extend(dump_yaml(value, indent + 2))
            else:
                lines.append(f"{pad}{key}: {yaml_scalar(value)}")
        return lines

    if isinstance(obj, list):
        for item in obj:
            if isinstance(item, dict):
                lines.append(f"{pad}-")
                dict_lines = dump_yaml(item, indent + 2)
                if dict_lines:
                    first = dict_lines[0]
                    lines[-1] = f"{pad}- {first.strip()}"
                    lines.extend(dict_lines[1:])
            elif isinstance(item, list):
                lines.append(f"{pad}-")
                lines.extend(dump_yaml(item, indent + 2))
            else:
                lines.append(f"{pad}- {yaml_scalar(item)}")
        return lines

    lines.append(f"{pad}{yaml_scalar(obj)}")
    return lines


def render_release_env(entries: list[dict]) -> str:
    lines = ["# Unified release workflow configuration.", ""]
    for entry in entries:
        key = entry["key"]
        kind = entry["type"]
        value = entry["value"]

        if kind == "bool":
            rendered = "true" if value else "false"
        elif kind == "json":
            rendered = "'" + json.dumps(value, separators=(",", ":")) + "'"
        elif kind == "string":
            rendered = str(value)
        else:
            raise ValueError(f"Unsupported release.env entry type: {kind}")

        lines.append(f"{key}={rendered}")

    lines.append("")
    return "\n".join(lines)


def render_yaml_document(data: Any) -> str:
    return "\n".join(dump_yaml(data)) + "\n"


def find_repo_config(manifest: dict, repo_name: str) -> dict:
    for repo in manifest["repositories"]:
        if repo["name"] == repo_name:
            return repo
    raise KeyError(f"Repository '{repo_name}' not found in manifest")


def write_if_needed(path: Path, content: str) -> None:
    if path.exists() and path.read_text(encoding="utf-8") == content:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def render_repo(repo_name: str, manifest: dict) -> None:
    repo = find_repo_config(manifest, repo_name)
    repo_root = ROOT / repo_name

    release_path = repo_root / ".github/release.env"
    release_content = render_release_env(repo.get("release_env", []))
    write_if_needed(release_path, release_content)

    dependabot_data = repo.get("dependabot")
    if dependabot_data is not None:
        dependabot_path = repo_root / ".github/dependabot.yml"
        dependabot_content = render_yaml_document(dependabot_data)
        write_if_needed(dependabot_path, dependabot_content)

    wrappers = repo.get("workflow_wrappers", {})
    if "ci" in wrappers:
        ci_path = repo_root / ".github/workflows/ci.yml"
        write_if_needed(ci_path, render_yaml_document(wrappers["ci"]))

    if "verify" in wrappers:
        verify_path = repo_root / ".github/workflows/verify.yml"
        write_if_needed(verify_path, render_yaml_document(wrappers["verify"]))

    pinned_sha = repo.get("pinned_std_sha")
    if pinned_sha:
        pin_path = repo_root / ".github/standards/bijux-std.sha"
        write_if_needed(pin_path, f"{pinned_sha}\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="Render release.env, dependabot.yml, and workflow wrappers from manifest")
    parser.add_argument("--manifest", default=str(MANIFEST_PATH), help="Path to manifest JSON")
    parser.add_argument("--repo", action="append", default=[], help="Repository name (repeatable)")
    args = parser.parse_args()

    manifest = json.loads(Path(args.manifest).read_text(encoding="utf-8"))
    repos = args.repo or [repo["name"] for repo in manifest["repositories"]]

    for repo_name in repos:
        render_repo(repo_name, manifest)


if __name__ == "__main__":
    main()
