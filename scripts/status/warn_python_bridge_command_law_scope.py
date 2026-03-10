#!/usr/bin/env python3
"""Warn CI if python bridge introduces new command-law logic."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BINDINGS = ROOT / "crates" / "bijux-cli-python" / "src" / "bindings.rs"

FORBIDDEN = (
    "match argv[",
    "if argv[",
    "argv.get(",
    "unknown route",
    "namespace rejection",
)


def main() -> int:
    if not BINDINGS.exists():
        print("::warning title=Python Bridge Law Scope::missing crates/bijux-cli-python/src/bindings.rs")
        return 0

    text = BINDINGS.read_text(encoding="utf-8").lower()
    hits = [marker for marker in FORBIDDEN if marker.lower() in text]
    if hits:
        print(
            "::warning title=Python Bridge Law Scope::"
            f"possible bridge-side command-law logic markers detected: {hits}"
        )
    else:
        print("python bridge command-law warning check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
