# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the plugin registry service.

This module specifies the `RegistryProtocol`, a formal interface that any
service responsible for managing the lifecycle of CLI plugins must implement.
This includes registering, retrieving, and invoking hooks on plugins.
"""

from __future__ import annotations

from typing import Any, Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class RegistryProtocol(Protocol):
    """Defines the contract for plugin management.

    This interface specifies the methods for registering, deregistering, and
    interacting with plugins, as well as for invoking plugin hooks.
    """

    def register(
        self,
        name: str,
        plugin: object,
        *,
        alias: str | None = None,
        version: str | None = None,
    ) -> None:
        """Registers a plugin with the registry.

        Args:
            name (str): The primary name of the plugin.
            plugin (object): The plugin object to register.
            alias (str | None): An optional alias for the plugin.
            version (str | None): An optional version string for the plugin.

        Returns:
            None:
        """
        ...

    def deregister(self, name: str) -> None:
        """Deregisters a plugin from the registry.

        Args:
            name (str): The name or alias of the plugin to deregister.

        Returns:
            None:
        """
        ...

    def get(self, name: str) -> object:
        """Retrieves a plugin by its name or alias.

        Args:
            name (str): The name or alias of the plugin.

        Returns:
            object: The registered plugin object.

        Raises:
            KeyError: If no plugin with the given name or alias is registered.
        """
        ...

    def has(self, name: str) -> bool:
        """Checks if a plugin exists in the registry.

        Args:
            name (str): The name or alias of the plugin.

        Returns:
            bool: True if the plugin is registered, False otherwise.
        """
        ...

    def names(self) -> list[str]:
        """Returns all registered plugin names.

        Returns:
            list[str]: A list of the primary names of all registered plugins.
        """
        ...

    def meta(self, name: str) -> dict[str, str]:
        """Returns metadata for a specific plugin.

        Args:
            name (str): The name or alias of the plugin.

        Returns:
            dict[str, str]: A dictionary containing the plugin's metadata, such
                as its version and alias.
        """
        ...

    async def call_hook(self, hook: str, *args: Any, **kwargs: Any) -> Any:
        """Invokes a hook on all registered plugins that implement it.

        Args:
            hook (str): The name of the hook to invoke.
            *args (Any): Positional arguments to pass to the hook.
            **kwargs (Any): Keyword arguments to pass to the hook.

        Returns:
            Any: The result of the hook invocation. The exact return type
                depends on the specific hook's definition and implementation.
        """
        ...
