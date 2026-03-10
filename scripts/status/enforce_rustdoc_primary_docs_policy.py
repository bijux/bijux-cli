#!/usr/bin/env python3
"""Enforce rustdoc-primary code-doc policy declarations."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    policy = ROOT / "docs" / "architecture" / "rustdoc_primary_code_docs_policy.md"
    text = policy.read_text(encoding="utf-8") if policy.exists() else ""

    failures: list[str] = []
    if "Rustdoc is the primary code documentation path" not in text:
        failures.append("missing primary rustdoc declaration")
    if "bijux dev cli rustdoc audit" not in text:
        failures.append("missing rustdoc audit maintainer command declaration")

    if failures:
        for failure in failures:
            print(f"rustdoc-policy-failure: {failure}")
        return 1

    print("rustdoc primary code docs policy satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
