# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Filesystem adapter interfaces and no-op defaults."""

from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path


class NoopFileSystem:
    """No-op filesystem adapter for tests or dry runs."""

    def read_text(self, path: Path) -> str:
        """Read text from a path (no-op returns empty string)."""
        return ""

    def write_text(self, path: Path, data: str) -> None:
        """Write text to a path (no-op)."""
        return None

    def exists(self, path: Path) -> bool:
        """Return False for all paths."""
        return False

    def iterdir(self, path: Path) -> Iterable[Path]:
        """Return an empty iterator."""
        return iter(())


__all__ = ["NoopFileSystem"]
