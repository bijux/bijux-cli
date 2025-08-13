# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Discovers and loads plugins distributed as Python packages.

This module provides the `load_entrypoints` function, which is responsible for
finding and loading plugins that have been installed into the Python
environment and registered under the `bijux_cli.plugins` entry point group.
This enables a distributable plugin ecosystem where plugins can be managed
via tools like `pip`.
"""

from __future__ import annotations

import asyncio
import contextlib
import importlib.metadata as im
import logging
import traceback
from typing import Any

from packaging.specifiers import SpecifierSet
from packaging.version import Version as PkgVersion

from bijux_cli.contracts import (
    ObservabilityProtocol,
    RegistryProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.di import DIContainer

_LOG = logging.getLogger("bijux_cli.plugin_loader")


def _iter_plugin_eps() -> list[im.EntryPoint]:
    """Returns all entry points in the 'bijux_cli.plugins' group.

    Returns:
        list[importlib.metadata.EntryPoint]: A list of all discovered plugin
            entry points.
    """
    try:
        eps = im.entry_points()
        return list(eps.select(group="bijux_cli.plugins"))
    except Exception:
        return []


def _compatible(plugin: Any) -> bool:
    """Determines if a plugin is compatible with the current CLI API version.

    Args:
        plugin (Any): The plugin instance or class, which should have a
            `requires_api_version` attribute string (e.g., ">=1.2.0").

    Returns:
        bool: True if the plugin's version requirement is met by the host CLI's
            API version, otherwise False.
    """
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
    """Discovers, loads, and registers all entry point-based plugins.

    This function iterates through all entry points in the 'bijux_cli.plugins'
    group. For each one, it attempts to load, instantiate, and register the
    plugin. It also performs an API version compatibility check and runs the
    plugin's `startup` hook if present.

    Note:
        All exceptions during the loading or startup of a single plugin are
        caught, logged, and reported via telemetry. A failed plugin will be
        deregistered and will not prevent other plugins from loading.

    Args:
        di (DIContainer | None): The dependency injection container. If None,
            the current global container is used.
        registry (RegistryProtocol | None): The plugin registry. If None, it is
            resolved from the DI container.

    Returns:
        None:
    """
    import bijux_cli

    di = di or DIContainer.current()
    registry = registry or di.resolve(RegistryProtocol)

    obs = di.resolve(ObservabilityProtocol, None)
    tel = di.resolve(TelemetryProtocol, None)

    for ep in _iter_plugin_eps():
        try:
            plugin_class = ep.load()
            plugin = plugin_class()

            if not _compatible(plugin):
                raise RuntimeError(
                    f"Plugin '{ep.name}' requires API {getattr(plugin, 'requires_api_version', 'N/A')}, "
                    f"host is {bijux_cli.api_version}"
                )

            for tgt in (plugin_class, plugin):
                raw = getattr(tgt, "version", None)
                if raw is not None and not isinstance(raw, str):
                    tgt.version = str(raw)

            registry.register(ep.name, plugin, version=plugin.version)

            startup = getattr(plugin, "startup", None)
            if asyncio.iscoroutinefunction(startup):
                await startup(di)
            elif callable(startup):
                startup(di)

            if obs:
                obs.log("info", f"Loaded plugin '{ep.name}'")
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


__all__ = ["load_entrypoints"]
