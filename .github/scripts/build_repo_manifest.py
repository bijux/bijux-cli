#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REPOS = [
    "bijux-atlas",
    "bijux-canon",
    "bijux-core",
    "bijux-masterclass",
    "bijux-pollenomics",
    "bijux-proteomics",
    "bijux-std",
    "bijux.github.io",
]


def parse_release_env(path: Path) -> list[dict]:
    entries: list[dict] = []
    if not path.exists():
        return entries

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        key, raw_value = line.split("=", 1)
        value = raw_value.strip()

        if value in {"true", "false"}:
            entries.append({"key": key, "type": "bool", "value": value == "true"})
            continue

        if value.startswith("'") and value.endswith("'") and len(value) >= 2:
            inner = value[1:-1]
            try:
                parsed_json = json.loads(inner)
            except json.JSONDecodeError:
                entries.append({"key": key, "type": "string", "value": value})
            else:
                entries.append({"key": key, "type": "json", "value": parsed_json})
            continue

        entries.append({"key": key, "type": "string", "value": value})

    return entries


def parse_yaml(path: Path) -> dict | None:
    if not path.exists():
        return None

    result = subprocess.run(
        [
            "ruby",
            "-ryaml",
            "-rjson",
            "-e",
            "puts JSON.generate(YAML.safe_load(File.read(ARGV[0]), aliases: false))",
            str(path),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def parse_text(path: Path) -> str | None:
    if not path.exists():
        return None
    return path.read_text(encoding="utf-8").strip()


def main() -> None:
    manifest: dict = {"version": 2, "repositories": []}

    for repo_name in REPOS:
        repo_path = ROOT / repo_name
        repo_entry: dict = {"name": repo_name}
        repo_entry["release_env"] = parse_release_env(repo_path / ".github/release.env")

        dependabot = parse_yaml(repo_path / ".github/dependabot.yml")
        if dependabot is not None:
            repo_entry["dependabot"] = dependabot

        wrappers: dict = {}
        ci_wrapper = parse_yaml(repo_path / ".github/workflows/ci.yml")
        if ci_wrapper is not None:
            wrappers["ci"] = ci_wrapper

        verify_wrapper = parse_yaml(repo_path / ".github/workflows/verify.yml")
        if verify_wrapper is not None:
            wrappers["verify"] = verify_wrapper

        if wrappers:
            repo_entry["workflow_wrappers"] = wrappers

        pinned_sha = parse_text(repo_path / ".github/standards/bijux-std.sha")
        if pinned_sha:
            repo_entry["pinned_std_sha"] = pinned_sha

        manifest["repositories"].append(repo_entry)

    out_path = ROOT / "bijux-std/.github/standards/repo-config.manifest.json"
    out_path.write_text(json.dumps(manifest, indent=2, sort_keys=False) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
