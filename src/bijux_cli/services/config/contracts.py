# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the application configuration service."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class ConfigProtocol(Protocol):
    """Defines the contract for application configuration management."""

    def load(self, path: str | Path | None = None) -> None: ...

    def get(self, key: str, default: Any = None) -> Any: ...

    def set(self, key: str, value: Any) -> None: ...

    def unset(self, key: str) -> None: ...

    def clear(self) -> None: ...

    def items(self) -> dict[str, Any]: ...

    def export(self) -> dict[str, Any]: ...

    def save(self, path: str | Path | None = None) -> None: ...

    def validate_env_file_if_present(self, path: str) -> None: ...


__all__ = ["ConfigProtocol"]
