#!/usr/bin/env python3
"""Enforce dev-cli release truth policy declarations."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    policy_path = ROOT / "docs" / "architecture" / "dev_cli_release_truth_policy.md"
    text = policy_path.read_text(encoding="utf-8") if policy_path.exists() else ""

    failures: list[str] = []
    required = [
        "dev cli release *",
        "bijux dev cli release status",
        "bijux dev cli release readiness",
        "bijux dev cli release evidence",
        "bijux dev cli release gaps",
        "dev_cli_release_truth_bundle.json",
    ]
    for needle in required:
        if needle not in text:
            failures.append(f"missing policy declaration: {needle}")

    if failures:
        for failure in failures:
            print(f"release-truth-policy-failure: {failure}")
        return 1

    print("dev cli release truth policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
