# SPDX-License-Identifier: Apache-2.0
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
from collections.abc import AsyncIterable, Callable
import contextlib
import importlib.metadata as im
import logging
import traceback
from types import MappingProxyType
from typing import Any

from injector import inject
from packaging.specifiers import SpecifierSet
from packaging.version import Version as PkgVersion
import pluggy

from bijux_cli.core.di import DIContainer
from bijux_cli.plugins.contracts import PluginState, RegistryProtocol
from bijux_cli.services.contracts import ObservabilityProtocol, TelemetryProtocol
from bijux_cli.services.errors import ServiceError

PRE_EXECUTE = "pre_execute"
POST_EXECUTE = "post_execute"
SPEC_VERSION = __import__("bijux_cli").version

hookspec = pluggy.HookspecMarker("bijux")


class CoreSpec:
    """Defines the core hook specifications for CLI plugins."""

    def __init__(self, dependency_injector: DIContainer) -> None:
        """Initialize with observability from DI."""
        self._log = dependency_injector.resolve(ObservabilityProtocol)

    @hookspec
    async def startup(self) -> None:
        """Hook called at startup."""
        self._log.log("debug", "Hook startup called", extra={})

    @hookspec
    async def shutdown(self) -> None:
        """Hook called at shutdown."""
        self._log.log("debug", "Hook shutdown called", extra={})

    @hookspec
    async def pre_execute(
        self, name: str, args: tuple[Any, ...], kwargs: dict[str, Any]
    ) -> None:
        """Hook called before command execution."""
        self._log.log(
            "debug",
            "Hook pre_execute called",
            extra={"name": name, "args": args, "kwargs": kwargs},
        )

    @hookspec
    async def post_execute(self, name: str, result: Any) -> None:
        """Hook called after command execution."""
        self._log.log(
            "debug",
            "Hook post_execute called",
            extra={"name": name, "result": repr(result)},
        )

    @hookspec
    def health(self) -> bool | str:
        """Hook used for health checks."""
        self._log.log("debug", "Hook health called", extra={})
        return True


def command_group(
    name: str,
    *,
    version: str | None = None,
) -> Callable[[str], Callable[[Callable[..., Any]], Callable[..., Any]]]:
    """Decorator factory for registering plugin subcommands under a group."""

    def with_sub(sub: str) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        if " " in sub:
            raise ValueError("subcommand may not contain spaces")
        full = f"{name} {sub}"

        def decorator(fn: Callable[..., Any]) -> Callable[..., Any]:
            try:
                di = DIContainer.current()
                reg: RegistryProtocol = di.resolve(RegistryProtocol)
            except KeyError as exc:
                raise RuntimeError("RegistryProtocol is not initialized") from exc

            reg.register(full, fn, alias=None, version=version)

            try:
                obs: ObservabilityProtocol = di.resolve(ObservabilityProtocol)
                obs.log(
                    "info",
                    "Registered command group",
                    extra={"cmd": full, "version": version},
                )
            except KeyError:
                pass

            try:
                tel: TelemetryProtocol = di.resolve(TelemetryProtocol)
                tel.event(
                    "command_group_registered", {"command": full, "version": version}
                )
            except KeyError:
                pass

            return fn

        return decorator

    return with_sub


def dynamic_choices(
    callback: Callable[[], list[str]],
    *,
    case_sensitive: bool = True,
) -> Callable[[Any, Any, str], list[str]]:
    """Creates a Typer completer from a callback function."""

    def completer(_ctx: Any, _param: Any, incomplete: str) -> list[str]:
        choices = callback()
        if case_sensitive:
            return [c for c in choices if c.startswith(incomplete)]
        return [c for c in choices if c.lower().startswith(incomplete.lower())]

    return completer


def _iter_plugin_eps() -> list[im.EntryPoint]:
    """Returns all entry points in the 'bijux_cli.plugins' group."""
    try:
        eps = im.entry_points()
        return list(eps.select(group="bijux_cli.plugins"))
    except Exception:
        return []


def _compatible(plugin: Any) -> bool:
    """Determines if a plugin is compatible with the current CLI API version."""
    import bijux_cli

    spec = getattr(plugin, "requires_api_version", ">=0.0.0")
    try:
        apiv = bijux_cli.api_version
        host_api_version = PkgVersion(str(apiv))
        return SpecifierSet(spec).contains(host_api_version)
    except Exception:
        return False


async def load_entrypoints(
    di: DIContainer | None = None,
    registry: RegistryProtocol | None = None,
) -> None:
    """Discovers, loads, and registers all entry point-based plugins."""
    import bijux_cli

    di = di or DIContainer.current()
    registry = registry or di.resolve(RegistryProtocol)

    obs = di.resolve(ObservabilityProtocol, None)
    tel = di.resolve(TelemetryProtocol, None)

    for ep in _iter_plugin_eps():
        try:
            plugin_class = await asyncio.to_thread(ep.load)
            plugin = await asyncio.to_thread(plugin_class)

            if not _compatible(plugin):
                raise RuntimeError(
                    f"Plugin '{ep.name}' requires API {getattr(plugin, 'requires_api_version', 'N/A')}, "
                    f"host is {bijux_cli.api_version}"
                )

            for tgt in (plugin_class, plugin):
                raw = getattr(tgt, "version", None)
                if raw is not None and not isinstance(raw, str):
                    tgt.version = str(raw)

            registry.transition(ep.name, PluginState.DISCOVERED)
            registry.transition(ep.name, PluginState.INSTALLED)
            registry.register(ep.name, plugin, alias=None, version=plugin.version)

            startup = getattr(plugin, "startup", None)
            if asyncio.iscoroutinefunction(startup):
                await startup(di)
            elif callable(startup):
                await asyncio.to_thread(startup, di)

            if obs:
                obs.log("info", f"Loaded plugin '{ep.name}'", extra={})
            if tel:
                tel.event("entrypoint_plugin_loaded", {"name": ep.name})

        except Exception as exc:
            with contextlib.suppress(Exception):
                registry.deregister(ep.name)

            if obs:
                obs.log(
                    "error",
                    f"Failed to load plugin '{ep.name}'",
                    extra={"trace": traceback.format_exc(limit=5)},
                )
            if tel:
                tel.event(
                    "entrypoint_plugin_failed", {"name": ep.name, "error": str(exc)}
                )

            _LOG.debug("Skipped plugin %s: %s", ep.name, exc, exc_info=True)


class Registry(RegistryProtocol):
    """A `pluggy`-based registry for managing CLI plugins.

    This class provides aliasing, metadata storage, and telemetry integration
    on top of the core `pluggy` plugin management system.

    Attributes:
        _telemetry (TelemetryProtocol): The telemetry service for events.
        _pm (pluggy.PluginManager): The underlying `pluggy` plugin manager.
        _plugins (dict): A mapping of canonical plugin names to plugin objects.
        _aliases (dict): A mapping of alias names to canonical plugin names.
        _meta (dict): A mapping of canonical plugin names to their metadata.
        mapping (MappingProxyType): A read-only view of the `_plugins` mapping.
    """

    @inject
    def __init__(self, telemetry: TelemetryProtocol):
        """Initializes the `Registry` service.

        Args:
            telemetry (TelemetryProtocol): The telemetry service for tracking
                registry events.
        """
        self._telemetry = telemetry
        self._pm = pluggy.PluginManager("bijux")
        self._pm.add_hookspecs(CoreSpec)
        self._plugins: dict[str, object] = {}
        self._aliases: dict[str, str] = {}
        self._meta: dict[str, dict[str, str]] = {}
        self._states: dict[str, PluginState] = {}
        self.mapping = MappingProxyType(self._plugins)

    def state(self, name: str) -> PluginState | None:
        """Return the lifecycle state for a plugin."""
        canonical = self._aliases.get(name, name)
        return self._states.get(canonical)

    def transition(self, name: str, state: PluginState) -> None:
        """Transition a plugin to a new lifecycle state."""
        allowed: dict[PluginState, set[PluginState]] = {
            PluginState.DISCOVERED: {
                PluginState.INSTALLED,
                PluginState.ACTIVE,
                PluginState.REMOVED,
            },
            PluginState.INSTALLED: {PluginState.ACTIVE, PluginState.INACTIVE},
            PluginState.ACTIVE: {PluginState.INACTIVE, PluginState.REMOVED},
            PluginState.INACTIVE: {PluginState.ACTIVE, PluginState.REMOVED},
            PluginState.REMOVED: set(),
        }
        canonical = self._aliases.get(name, name)
        current = self._states.get(canonical)
        if current is None:
            self._states[canonical] = state
            return
        if state not in allowed.get(current, set()):
            raise ServiceError(
                f"Invalid plugin state transition {current.value} -> {state.value}",
                http_status=400,
            )
        self._states[canonical] = state

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
        self.transition(name, PluginState.ACTIVE)
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
        self._states[canonical] = PluginState.REMOVED
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


__all__ = [
    "Registry",
    "CoreSpec",
    "SPEC_VERSION",
    "PRE_EXECUTE",
    "POST_EXECUTE",
    "command_group",
    "dynamic_choices",
    "load_entrypoints",
    "_iter_plugin_eps",
    "_compatible",
]
_LOG = logging.getLogger("bijux_cli.plugin_loader")
