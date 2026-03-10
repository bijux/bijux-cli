#!/usr/bin/env python3
"""Enforce test-first domain gates for plugin, flag, and determinism areas."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def changed_test_files(base_rev: str) -> list[Path]:
    cmd = ["git", "diff", "--name-only", f"{base_rev}..HEAD", "--", "crates/*/tests/*.rs"]
    out = subprocess.run(cmd, cwd=ROOT, check=False, text=True, capture_output=True)
    files = []
    for line in out.stdout.splitlines():
        path = (ROOT / line.strip())
        if line.strip().endswith(".rs") and path.exists():
            files.append(path)
    return files


def file_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="ignore").lower()


def in_domain(path: Path, name: str) -> bool:
    text = str(path).lower()
    if name == "plugin":
        return "plugin" in text
    if name == "flag":
        return "flag" in text or "parser" in text
    if name == "determinism":
        return "deterministic" in text
    return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--enforce", action="store_true")
    parser.add_argument("--base-rev", default="HEAD~1")
    args = parser.parse_args()

    failures: list[str] = []

    files = changed_test_files(args.base_rev)
    for path in files:
        text = file_text(path)
        if any(in_domain(path, d) for d in ("plugin", "flag", "determinism")):
            if "test_type:" not in text:
                failures.append(f"missing test_type tag: {path.relative_to(ROOT)}")

        if in_domain(path, "plugin") and ("fn " in text) and ("failure" not in text and "rollback" not in text):
            failures.append(
                f"plugin test file lacks failure-path/rollback evidence: {path.relative_to(ROOT)}"
            )

        if in_domain(path, "flag") and ("fn " in text) and (
            "precedence" not in text and "conflict" not in text
        ):
            failures.append(
                f"flag test file lacks precedence/conflict evidence: {path.relative_to(ROOT)}"
            )

        if in_domain(path, "determinism") and ("fn " in text) and (
            "for _ in" not in text and "across runs" not in text and "repeat" not in text
        ):
            failures.append(
                f"determinism test file lacks repeated-run proof evidence: {path.relative_to(ROOT)}"
            )

    if failures:
        print("TEST-FIRST DOMAIN GATE FAILURES:")
        for item in failures:
            print(f" - {item}")
        return 1 if args.enforce else 0

    print("Test-first domain gates passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
