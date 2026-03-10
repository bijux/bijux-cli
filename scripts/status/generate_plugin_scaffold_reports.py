#!/usr/bin/env python3
"""Generate plugin scaffold minimalism artifacts."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"
SNAPSHOT_DIR = ROOT / "crates" / "bijux-cli" / "tests" / "snapshots"


def stable_generated_at() -> str:
    source_date_epoch = subprocess.run(
        ["sh", "-lc", "printf %s \"${SOURCE_DATE_EPOCH:-}\""],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if source_date_epoch.isdigit():
        return datetime.fromtimestamp(int(source_date_epoch), tz=timezone.utc).isoformat()
    return "1970-01-01T00:00:00+00:00"


def read_snapshot(name: str) -> list[str]:
    text = (SNAPSHOT_DIR / name).read_text(encoding="utf-8")
    return [line.strip() for line in text.splitlines() if line.strip()]


def write_json(name: str, payload: dict[str, Any]) -> None:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    path = STATUS_DIR / name
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_text(name: str, lines: list[str]) -> None:
    STATUS_DIR.mkdir(parents=True, exist_ok=True)
    path = STATUS_DIR / name
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    generated_at = stable_generated_at()
    python_files = read_snapshot("plugin_scaffold_python_minimal_files.txt")
    rust_files = read_snapshot("plugin_scaffold_rust_minimal_files.txt")

    by_kind = {
        "python": python_files,
        "rust": rust_files,
    }

    justifications = {
        "python": {
            "plugin.manifest.json": {
                "classification": "essential",
                "reason": "required for install, namespace validation, and lifecycle commands",
            },
            "plugin.py": {
                "classification": "essential",
                "reason": "runtime entrypoint for delegated python plugins",
            },
        },
        "rust": {
            "plugin.manifest.json": {
                "classification": "essential",
                "reason": "required for install, namespace validation, and lifecycle commands",
            },
            "src/lib.rs": {
                "classification": "essential",
                "reason": "runtime entrypoint module for delegated rust plugins",
            },
        },
    }

    decorative_files = ["README.md", "pyproject.toml", "Cargo.toml", ".gitignore"]
    decorative_present = {
        kind: [path for path in files if path in decorative_files] for kind, files in by_kind.items()
    }

    python_set = set(python_files)
    rust_set = set(rust_files)

    write_json(
        "plugin_scaffold_python_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_plugin_scaffold_reports.py",
            "kind": "python",
            "files": python_files,
            "count": len(python_files),
        },
    )
    write_json(
        "plugin_scaffold_rust_inventory.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_plugin_scaffold_reports.py",
            "kind": "rust",
            "files": rust_files,
            "count": len(rust_files),
        },
    )
    write_json(
        "plugin_scaffold_diff.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_plugin_scaffold_reports.py",
            "shared": sorted(python_set & rust_set),
            "python_only": sorted(python_set - rust_set),
            "rust_only": sorted(rust_set - python_set),
        },
    )
    write_json(
        "plugin_scaffold_non_behavioral_files.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_plugin_scaffold_reports.py",
            "decorative_candidates": decorative_files,
            "present_in_scaffold": decorative_present,
            "summary": "decorative files are excluded from minimal scaffold outputs",
        },
    )
    write_json(
        "plugin_scaffold_file_justification.json",
        {
            "generated_at": generated_at,
            "generator": "scripts/status/generate_plugin_scaffold_reports.py",
            "classification_values": ["essential", "helpful", "removable"],
            "files": justifications,
            "freeze_rule": "every scaffolded file must have a justification and decorative outputs stay excluded",
        },
    )

    write_text(
        "plugin_scaffold_minimalism_summary.txt",
        [
            "Plugin scaffold minimalism summary",
            f"Generated at: {generated_at}",
            f"Python files ({len(python_files)}): {', '.join(python_files)}",
            f"Rust files ({len(rust_files)}): {', '.join(rust_files)}",
            "Decorative files excluded: README.md, pyproject.toml, Cargo.toml, .gitignore",
            "Policy: every scaffolded file must carry explicit justification",
        ],
    )

    print("wrote plugin scaffold minimalism artifacts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
