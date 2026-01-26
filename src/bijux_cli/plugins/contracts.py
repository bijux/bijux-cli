# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Plugin-level protocol contracts."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
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

    def state(self, name: str) -> PluginState | None:
        """Return plugin lifecycle state."""
        ...

    def transition(self, name: str, state: PluginState) -> None:
        """Move a plugin to a lifecycle state."""
        ...

    async def call_hook(self, hook: str, *args: Any, **kwargs: Any) -> Any:
        """Invoke a hook on all plugins."""
        ...


@dataclass(frozen=True)
class PluginConfig:
    """Configuration for plugin discovery and activation."""

    enabled: bool
    allow_entrypoints: bool


__all__ = ["PluginConfig", "PluginState", "RegistryProtocol"]


class PluginState(str, Enum):
    """Lifecycle states for plugins."""

    DISCOVERED = "discovered"
    INSTALLED = "installed"
    ACTIVE = "active"
    INACTIVE = "inactive"
    REMOVED = "removed"
