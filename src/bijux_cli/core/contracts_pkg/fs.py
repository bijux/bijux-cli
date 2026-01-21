# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for filesystem access adapters."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Protocol, runtime_checkable


@runtime_checkable
class FileSystemProtocol(Protocol):
    """Defines the contract for filesystem access."""

    def read_text(self, path: str) -> str: ...

    def write_text(self, path: str, data: str) -> None: ...

    def exists(self, path: str) -> bool: ...

    def iterdir(self, path: str) -> Iterable[str]: ...
