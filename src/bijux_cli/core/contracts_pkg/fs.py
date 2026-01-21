# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for filesystem access adapters."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Protocol, runtime_checkable


@runtime_checkable
class FileSystemProtocol(Protocol):
    """Defines the contract for filesystem access."""

    def read_text(self, path: str) -> str:
        """Read text content from a path."""
        ...

    def write_text(self, path: str, data: str) -> None:
        """Write text content to a path."""
        ...

    def exists(self, path: str) -> bool:
        """Return True if a path exists."""
        ...

    def iterdir(self, path: str) -> Iterable[str]:
        """Return directory entries for a path."""
        ...
