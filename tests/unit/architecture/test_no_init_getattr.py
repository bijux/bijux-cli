# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Architecture tests for package init modules."""

from __future__ import annotations

from pathlib import Path


def test_no_getattr_in_init_files() -> None:
    """Package __init__ modules should not define __getattr__ shims."""
    repo_root = Path(__file__).resolve().parents[3]
    for path in (repo_root / "src" / "bijux_cli").rglob("__init__.py"):
        text = path.read_text(encoding="utf-8")
        assert "__getattr__" not in text, f"__getattr__ found in {path}"
