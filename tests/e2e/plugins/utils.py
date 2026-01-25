# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin helpers for E2E tests."""

from __future__ import annotations

from pathlib import Path


def write_dummy_plugin(path: Path, name: str = "dummy_plugin") -> Path:
    """Create a minimal local plugin directory and return its path."""
    path.mkdir(parents=True, exist_ok=True)
    (path / "plugin.py").write_text(
        "\n".join(
            [
                "def setup() -> None:",
                "    return None",
                "",
            ]
        ),
        encoding="utf-8",
    )
    (path / "plugin.json").write_text(
        "\n".join(
            [
                "{",
                f'  "name": "{name}",',
                '  "version": "0.1.0",',
                '  "bijux_cli_version": ">=0.0.0",',
                '  "enabled": true',
                "}",
            ]
        ),
        encoding="utf-8",
    )
    return path
