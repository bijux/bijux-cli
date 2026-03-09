#!/usr/bin/env python3
"""Enforce test quality gates for weak/filler policy."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
AUDIT = ROOT / "artifacts" / "status" / "test_quality_audit.json"
CHECKLIST = ROOT / "docs" / "architecture" / "test-review-checklist.md"
POLICY = ROOT / "docs" / "architecture" / "test-policy.md"

REQUIRED_POLICY_LINES = [
    "at least one failure-path test",
    "stateful commands: at least one filesystem-failure test",
    "parser features: at least one malformed-input test",
    "plugin lifecycle features: at least one rollback test",
    "No vanity test counts",
]


def read_text(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8", errors="ignore")


def main() -> int:
    if not AUDIT.exists():
        print("TEST QUALITY FAILURE: missing artifacts/status/test_quality_audit.json")
        return 1

    audit = json.loads(AUDIT.read_text(encoding="utf-8"))
    tests = audit.get("tests", [])

    tagged_filler = []
    for row in tests:
        path = ROOT / row.get("path", "")
        text = read_text(path).lower()
        if "test_tag: filler" in text or "test-tag: filler" in text:
            tagged_filler.append(row.get("path", ""))

    if tagged_filler:
        print("TEST QUALITY FAILURE: filler-tagged tests are not allowed")
        for item in tagged_filler:
            print(f" - {item}")
        return 1

    policy = read_text(POLICY)
    missing_lines = [line for line in REQUIRED_POLICY_LINES if line not in policy]
    if missing_lines:
        print("TEST QUALITY FAILURE: test policy is missing required rule text")
        for line in missing_lines:
            print(f" - {line}")
        return 1

    checklist = read_text(CHECKLIST)
    if "failure-path" not in checklist or "rollback" not in checklist:
        print("TEST QUALITY FAILURE: test review checklist is incomplete")
        return 1

    print("Test quality gates passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
