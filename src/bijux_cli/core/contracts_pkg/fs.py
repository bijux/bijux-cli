# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for filesystem access adapters."""

from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path
from typing import Protocol, runtime_checkable


@runtime_checkable
class FileSystemProtocol(Protocol):
    """Defines the contract for filesystem access."""

    def read_text(self, path: Path) -> str: ...

    def write_text(self, path: Path, data: str) -> None: ...

    def exists(self, path: Path) -> bool: ...

    def iterdir(self, path: Path) -> Iterable[Path]: ...
