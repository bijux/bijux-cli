#!/usr/bin/env python3
"""Fail when new public APIs are introduced without explicit justification."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ALLOWLIST = ROOT / ".github" / "public_api_allowlist.txt"

PUBLIC_ITEM_RE = re.compile(
    r"^\+\s*pub\s+(?:\([^\)]*\)\s+)?(?:fn|struct|enum|trait|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)


def run(*args: str) -> str:
    proc = subprocess.run(args, cwd=ROOT, text=True, capture_output=True)
    if proc.returncode != 0:
        return ""
    return proc.stdout


def load_allowlist() -> dict[str, str]:
    if not ALLOWLIST.exists():
        return {}
    rows: dict[str, str] = {}
    for line in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if ":" not in stripped:
            continue
        symbol, reason = stripped.split(":", 1)
        rows[symbol.strip()] = reason.strip()
    return rows


def main() -> int:
    diff = run("git", "diff", "--unified=0", "--", "crates/**/src/*.rs", "crates/**/src/**/*.rs")
    if not diff:
        return 0

    added_symbols: set[str] = set()
    for line in diff.splitlines():
        match = PUBLIC_ITEM_RE.match(line)
        if match:
            added_symbols.add(match.group(1))

    if not added_symbols:
        return 0

    allowlist = load_allowlist()
    failures: list[str] = []
    for symbol in sorted(added_symbols):
        reason = allowlist.get(symbol, "")
        if not reason:
            failures.append(
                f"new public API `{symbol}` is missing justification in .github/public_api_allowlist.txt"
            )

    for failure in failures:
        print(f"PUBLIC API POLICY FAILURE: {failure}")
    return 2 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())

