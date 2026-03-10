#!/usr/bin/env python3
"""Enforce plugin scaffold minimalism and file-justification rules."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATUS_DIR = ROOT / "artifacts" / "status"
ALLOWED = {"essential", "helpful", "removable"}


def read_json(name: str) -> dict:
    path = STATUS_DIR / name
    if not path.exists():
        raise FileNotFoundError(path)
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    py_inv = read_json("plugin_scaffold_python_inventory.json")
    rs_inv = read_json("plugin_scaffold_rust_inventory.json")
    non_behavioral = read_json("plugin_scaffold_non_behavioral_files.json")
    justification = read_json("plugin_scaffold_file_justification.json")

    failures: list[str] = []

    files_by_kind = {
        "python": py_inv.get("files", []),
        "rust": rs_inv.get("files", []),
    }
    meta_by_kind = justification.get("files", {})

    for kind, files in files_by_kind.items():
        if not isinstance(files, list) or not files:
            failures.append(f"{kind} scaffold inventory is empty")
            continue
        meta = meta_by_kind.get(kind, {})
        for path in files:
            record = meta.get(path)
            if not isinstance(record, dict):
                failures.append(f"missing justification for {kind}:{path}")
                continue
            classification = record.get("classification")
            reason = str(record.get("reason", "")).strip()
            if classification not in ALLOWED:
                failures.append(f"invalid classification for {kind}:{path}: {classification}")
            if not reason:
                failures.append(f"missing reason for {kind}:{path}")

    present = non_behavioral.get("present_in_scaffold", {})
    for kind in ("python", "rust"):
        found = present.get(kind, [])
        if found:
            failures.append(f"decorative files still present in {kind} scaffold: {', '.join(found)}")

    if failures:
        print("PLUGIN SCAFFOLD POLICY VIOLATION")
        for item in failures:
            print(f" - {item}")
        return 1

    print("Plugin scaffold policy passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
