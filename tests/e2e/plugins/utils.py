# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Helpers for building dummy plugins for E2E tests."""

from __future__ import annotations

from pathlib import Path


def write_dummy_plugin(root: Path, *, name: str) -> Path:
    """Create a minimal local plugin layout for lifecycle tests."""
    root.mkdir(parents=True, exist_ok=True)
    src_dir = root / "src" / name
    src_dir.mkdir(parents=True, exist_ok=True)

    (root / "plugin.json").write_text(
        "\n".join(
            [
                "{",
                f'  "name": "{name}",',
                '  "schema_version": "1",',
                '  "version": "0.1.0",',
                '  "bijux_cli_version": ">=0.0.0",',
                '  "enabled": true',
                "}",
            ]
        ),
        encoding="utf-8",
    )
    (root / "plugin.py").write_text("def setup():\n    return None\n", encoding="utf-8")
    (src_dir / "__init__.py").write_text("", encoding="utf-8")
    (src_dir / "plugin.py").write_text(
        "def setup():\n    return None\n", encoding="utf-8"
    )
    return root
