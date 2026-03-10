#!/usr/bin/env python3
"""Warn (without failing) when newly-added public APIs have no external consumer."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PUBLIC_FN_RE = re.compile(r"^\+\s*pub\s+(?:\([^\)]*\)\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b")


def run(*args: str) -> str:
    proc = subprocess.run(args, cwd=ROOT, text=True, capture_output=True)
    if proc.returncode != 0:
        return ""
    return proc.stdout


def main() -> int:
    diff = run("git", "diff", "--unified=0", "--", "crates/**/src/*.rs", "crates/**/src/**/*.rs")
    if not diff:
        return 0

    candidates: set[str] = set()
    for line in diff.splitlines():
        m = PUBLIC_FN_RE.match(line)
        if m:
            candidates.add(m.group(1))

    if not candidates:
        return 0

    tree = run("rg", "-n", "\\b(" + "|".join(sorted(candidates)) + ")\\b", "crates")
    hits: dict[str, int] = {name: 0 for name in candidates}
    for row in tree.splitlines():
        for name in candidates:
            if re.search(rf"\b{name}\b", row):
                hits[name] += 1

    warned = False
    for name, count in sorted(hits.items()):
        # declaration + at least one call site is the minimum useful signal.
        if count <= 1:
            warned = True
            print(
                f"::warning title=Unused public API candidate::new public function `{name}` has no detected consumer; keep private or document the reason"
            )

    if not warned:
        print("new public API usage check: no unused additions detected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
