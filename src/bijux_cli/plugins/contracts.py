# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Plugin-level protocol contracts."""

from __future__ import annotations

from typing import Any, Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class RegistryProtocol(Protocol):
    """Contract for plugin registry management."""

    def register(
        self,
        name: str,
        plugin: object,
        *,
        alias: str | None,
        version: str | None,
    ) -> None:
        """Register a plugin."""
        ...

    def deregister(self, name: str) -> None:
        """Remove a plugin."""
        ...

    def get(self, name: str) -> object:
        """Retrieve a plugin by name or alias."""
        ...

    def has(self, name: str) -> bool:
        """Check if a plugin exists."""
        ...

    def names(self) -> list[str]:
        """List registered plugin names."""
        ...

    def meta(self, name: str) -> dict[str, str]:
        """Return plugin metadata."""
        ...

    async def call_hook(self, hook: str, *args: Any, **kwargs: Any) -> Any:
        """Invoke a hook on all plugins."""
        ...


__all__ = ["RegistryProtocol"]
