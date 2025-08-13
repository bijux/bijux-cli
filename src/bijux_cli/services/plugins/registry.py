# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a concrete plugin registry service using the `pluggy` framework.

This module defines the `Registry` class, which implements the
`RegistryProtocol`. It serves as the central manager for the entire plugin
lifecycle, including registration, aliasing, metadata storage, and the
invocation of plugin hooks. It is built on top of the `pluggy` library to
provide a robust and extensible plugin architecture.
"""

from __future__ import annotations

import asyncio
from collections.abc import AsyncIterable
from types import MappingProxyType
from typing import Any

from injector import inject
import pluggy

from bijux_cli.contracts import RegistryProtocol
from bijux_cli.core.exceptions import ServiceError
from bijux_cli.infra.telemetry import LoggingTelemetry


class Registry(RegistryProtocol):
    """A `pluggy`-based registry for managing CLI plugins.

    This class provides aliasing, metadata storage, and telemetry integration
    on top of the core `pluggy` plugin management system.

    Attributes:
        _telemetry (LoggingTelemetry): The telemetry service for events.
        _pm (pluggy.PluginManager): The underlying `pluggy` plugin manager.
        _plugins (dict): A mapping of canonical plugin names to plugin objects.
        _aliases (dict): A mapping of alias names to canonical plugin names.
        _meta (dict): A mapping of canonical plugin names to their metadata.
        mapping (MappingProxyType): A read-only view of the `_plugins` mapping.
    """

    @inject
    def __init__(self, telemetry: LoggingTelemetry):
        """Initializes the `Registry` service.

        Args:
            telemetry (LoggingTelemetry): The telemetry service for tracking
                registry events.
        """
        self._telemetry = telemetry
        self._pm = pluggy.PluginManager("bijux")
        from bijux_cli.services.plugins.hooks import CoreSpec

        self._pm.add_hookspecs(CoreSpec)
        self._plugins: dict[str, object] = {}
        self._aliases: dict[str, str] = {}
        self._meta: dict[str, dict[str, str]] = {}
        self.mapping = MappingProxyType(self._plugins)

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
            name (str): The canonical name of the plugin.
            plugin (object): The plugin object to register.
            alias (str | None): An optional alias for the plugin.
            version (str | None): An optional version string for the plugin.

        Returns:
            None:

        Raises:
            ServiceError: If the name, alias, or plugin object is already
                registered, or if the underlying `pluggy` registration fails.
        """
        if name in self._plugins:
            raise ServiceError(f"Plugin {name!r} already registered", http_status=400)
        if plugin in self._plugins.values():
            raise ServiceError(
                "Plugin object already registered under a different name",
                http_status=400,
            )
        if alias and (alias in self._plugins or alias in self._aliases):
            raise ServiceError(f"Alias {alias!r} already in use", http_status=400)
        try:
            self._pm.register(plugin, name)
        except ValueError as error:
            raise ServiceError(
                f"Pluggy failed to register {name}: {error}", http_status=500
            ) from error
        self._plugins[name] = plugin
        self._meta[name] = {"version": version or "unknown"}
        if alias:
            self._aliases[alias] = name
        try:
            self._telemetry.event(
                "registry_plugin_registered",
                {"name": name, "alias": alias, "version": version},
            )
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed",
                {"operation": "register", "error": str(error)},
            )

    def deregister(self, name: str) -> None:
        """Deregisters a plugin from the registry.

        Args:
            name (str): The name or alias of the plugin to deregister.

        Returns:
            None:

        Raises:
            ServiceError: If the underlying `pluggy` deregistration fails.
        """
        canonical = self._aliases.get(name, name)
        plugin = self._plugins.pop(canonical, None)
        if not plugin:
            return
        try:
            self._pm.unregister(plugin)
        except ValueError as error:
            raise ServiceError(
                f"Pluggy failed to deregister {canonical}: {error}", http_status=500
            ) from error
        self._meta.pop(canonical, None)
        self._aliases = {a: n for a, n in self._aliases.items() if n != canonical}
        try:
            self._telemetry.event("registry_plugin_deregistered", {"name": canonical})
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed",
                {"operation": "deregister", "error": str(error)},
            )

    def get(self, name: str) -> object:
        """Retrieves a plugin by its name or alias.

        Args:
            name (str): The name or alias of the plugin to retrieve.

        Returns:
            object: The registered plugin object.

        Raises:
            ServiceError: If the plugin is not found.
        """
        canonical = self._aliases.get(name, name)
        try:
            plugin = self._plugins[canonical]
        except KeyError as key_error:
            try:
                self._telemetry.event(
                    "registry_plugin_retrieve_failed",
                    {"name": name, "error": str(key_error)},
                )
            except RuntimeError as telemetry_error:
                self._telemetry.event(
                    "registry_telemetry_failed",
                    {"operation": "retrieve_failed", "error": str(telemetry_error)},
                )
            raise ServiceError(
                f"Plugin {name!r} not found", http_status=404
            ) from key_error
        try:
            self._telemetry.event("registry_plugin_retrieved", {"name": canonical})
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed",
                {"operation": "retrieve", "error": str(error)},
            )
        return plugin

    def names(self) -> list[str]:
        """Returns a list of all registered plugin names.

        Returns:
            list[str]: A list of the canonical names of all registered plugins.
        """
        names = list(self._plugins.keys())
        try:
            self._telemetry.event("registry_list", {"names": names})
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed", {"operation": "list", "error": str(error)}
            )
        return names

    def has(self, name: str) -> bool:
        """Checks if a plugin is registered under a given name or alias.

        Args:
            name (str): The name or alias of the plugin to check.

        Returns:
            bool: True if the plugin is registered, otherwise False.
        """
        exists = name in self._plugins or name in self._aliases
        try:
            self._telemetry.event("registry_contains", {"name": name, "result": exists})
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed",
                {"operation": "contains", "error": str(error)},
            )
        return exists

    def meta(self, name: str) -> dict[str, str]:
        """Retrieves metadata for a specific plugin.

        Args:
            name (str): The name or alias of the plugin.

        Returns:
            dict[str, str]: A dictionary containing the plugin's metadata.
        """
        canonical = self._aliases.get(name, name)
        info = dict(self._meta.get(canonical, {}))
        try:
            self._telemetry.event("registry_meta_retrieved", {"name": canonical})
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed",
                {"operation": "meta_retrieved", "error": str(error)},
            )
        return info

    async def call_hook(self, hook: str, *args: Any, **kwargs: Any) -> list[Any]:
        """Invokes a hook on all registered plugins that implement it.

        This method handles results from multiple plugins, awaiting any results
        that are coroutines.

        Args:
            hook (str): The name of the hook to invoke.
            *args (Any): Positional arguments to pass to the hook.
            **kwargs (Any): Keyword arguments to pass to the hook.

        Returns:
            list[Any]: A list containing the results from all hook
                implementations that did not return `None`.

        Raises:
            ServiceError: If the specified hook does not exist.
        """
        try:
            hook_fn = getattr(self._pm.hook, hook)
            results = hook_fn(*args, **kwargs)
        except AttributeError as error:
            raise ServiceError(f"Hook {hook!r} not found", http_status=404) from error
        collected = []
        if isinstance(results, AsyncIterable):
            async for result in results:
                if asyncio.iscoroutine(result):
                    collected.append(await result)
                elif result is not None:
                    collected.append(result)
        else:
            for result in results:
                if asyncio.iscoroutine(result):
                    collected.append(await result)
                elif result is not None:
                    collected.append(result)
        try:
            self._telemetry.event("registry_hook_called", {"hook": hook})
        except RuntimeError as error:
            self._telemetry.event(
                "registry_telemetry_failed",
                {"operation": "hook_called", "error": str(error)},
            )
        return collected


__all__ = ["Registry"]
