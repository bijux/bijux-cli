# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Filesystem adapter interfaces and no-op defaults."""

from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path
from typing import Protocol


class FileSystem(Protocol):
    """Minimal filesystem adapter used by services."""

    def read_text(self, path: Path) -> str: ...

    def write_text(self, path: Path, data: str) -> None: ...

    def exists(self, path: Path) -> bool: ...

    def iterdir(self, path: Path) -> Iterable[Path]: ...


class NoopFileSystem:
    """No-op filesystem adapter for tests or dry runs."""

    def read_text(self, path: Path) -> str:
        return ""

    def write_text(self, path: Path, data: str) -> None:
        return None

    def exists(self, path: Path) -> bool:
        return False

    def iterdir(self, path: Path) -> Iterable[Path]:
        return iter(())


__all__ = ["FileSystem", "NoopFileSystem"]
