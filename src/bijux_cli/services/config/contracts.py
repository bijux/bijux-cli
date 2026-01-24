# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the application configuration service."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class ConfigProtocol(Protocol):
    """Defines the contract for application configuration management."""

    def load(self, path: str | Path | None = None) -> None:
        """Load configuration from a file path."""
        ...

    def reload(self) -> None:
        """Reload configuration from the last loaded path."""
        ...

    def get(self, key: str, default: Any = None) -> Any:
        """Retrieve a configuration value."""
        ...

    def set(self, key: str, value: Any) -> None:
        """Set a configuration value."""
        ...

    def unset(self, key: str) -> None:
        """Remove a configuration key."""
        ...

    def clear(self) -> None:
        """Clear all configuration values."""
        ...

    def all(self) -> dict[str, str]:
        """Return all configuration values."""
        ...

    def list_keys(self) -> list[str]:
        """List configuration keys."""
        ...

    def export(self, path: str | Path, out_format: str | None = None) -> None:
        """Export configuration to a target path."""
        ...

    def save(self) -> None:
        """Persist configuration to disk."""
        ...


__all__ = ["ConfigProtocol"]
