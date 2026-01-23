# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

from __future__ import annotations

from pathlib import Path


def test_root_package_minimal_files() -> None:
    """Ensure root package only exposes __init__.py and py.typed files."""
    root = Path(__file__).resolve().parents[3] / "src" / "bijux_cli"
    allowed = {"__init__.py", "py.typed"}
    files = [
        p.name for p in root.iterdir() if p.is_file() and not p.name.startswith(".")
    ]
    extras = sorted(name for name in files if name not in allowed)
    assert extras == [], f"Unexpected root files: {extras}"
