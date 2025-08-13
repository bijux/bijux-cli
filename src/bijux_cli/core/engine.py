# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the core runtime engine for the Bijux CLI.

This module defines the `Engine` class, which is responsible for orchestrating
the application's runtime environment after initial setup. Its key
responsibilities include:

    * Initializing and registering all default services with the Dependency
        Injection (DI) container.
    * Discovering, loading, and registering all external plugins.
    * Providing a central method for dispatching commands to plugins.
    * Managing the graceful shutdown of services.

The engine acts as the bridge between the CLI command layer and the
underlying services and plugins.
"""

from __future__ import annotations

import asyncio
import inspect
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from bijux_cli.core.di import DIContainer

from bijux_cli.contracts import ConfigProtocol, RegistryProtocol
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import CommandError
from bijux_cli.infra.observability import Observability
from bijux_cli.services import register_default_services
from bijux_cli.services.history import History
from bijux_cli.services.plugins import get_plugins_dir, load_plugin


class Engine:
    """Orchestrates the CLI's runtime services and plugin lifecycle.

    Attributes:
        _di (DIContainer): The dependency injection container.
        _debug (bool): The debug mode flag.
        _format (OutputFormat): The default output format.
        _quiet (bool): The quiet mode flag.
        _registry (RegistryProtocol): The plugin registry service.
    """

    def __init__(
        self,
        di: Any = None,
        *,
        debug: bool = False,
        fmt: OutputFormat = OutputFormat.JSON,
        quiet: bool = False,
    ) -> None:
        """Initializes the engine and its core services.

        This sets up the DI container, registers default services, and loads
        all discoverable plugins.

        Args:
            di (Any, optional): An existing dependency injection container. If
                None, the global singleton instance is used. Defaults to None.
            debug (bool): If True, enables debug mode for services.
            fmt (OutputFormat): The default output format for services.
            quiet (bool): If True, suppresses output from services.
        """
        from bijux_cli.core.di import DIContainer

        self._di = di or DIContainer.current()
        self._debug = debug
        self._format = fmt
        self._quiet = quiet
        self._di.register(Observability, lambda: Observability(debug=debug))
        register_default_services(self._di, debug=debug, output_format=fmt, quiet=quiet)
        self._di.register(Engine, self)
        self._registry: RegistryProtocol = self._di.resolve(RegistryProtocol)
        self._register_plugins()

    async def run_command(self, name: str, *args: Any, **kwargs: Any) -> Any:
        """Executes a plugin's command with a configured timeout.

        Args:
            name (str): The name of the command or plugin to run.
            *args (Any): Positional arguments to pass to the plugin's `execute`
                method.
            **kwargs (Any): Keyword arguments to pass to the plugin's `execute`
                method.

        Returns:
            Any: The result of the command's execution.

        Raises:
            CommandError: If the plugin is not found, its `execute` method
                is invalid, or if it fails during execution.
        """
        plugin = self._registry.get(name)
        execute = getattr(plugin, "execute", None)
        if not callable(execute):
            raise CommandError(
                f"Plugin '{name}' has no callable 'execute' method.", http_status=404
            )
        if not inspect.iscoroutinefunction(execute):
            raise CommandError(
                f"Plugin '{name}' 'execute' is not async/coroutine.", http_status=400
            )
        try:
            return await asyncio.wait_for(execute(*args, **kwargs), self._timeout())
        except Exception as exc:  # pragma: no cover
            raise CommandError(f"Failed to run plugin '{name}': {exc}") from exc

    async def run_repl(self) -> None:
        """Runs the interactive shell (REPL).

        Note: This is a placeholder for future REPL integration.
        """
        pass

    async def shutdown(self) -> None:
        """Gracefully shuts down the engine and all resolved services.

        This method orchestrates the termination sequence for the application's
        runtime. It first attempts to flush any buffered command history to
        disk and then proceeds to shut down the main dependency injection
        container, which in turn cleans up all resolved services.

        Returns:
            None:
        """
        try:
            self._di.resolve(History).flush()
        except KeyError:
            pass
        finally:
            await self._di.shutdown()

    def _register_plugins(self) -> None:
        """Discovers, loads, and registers all plugins from the filesystem.

        This method scans the plugins directory for valid plugin subdirectories.
        For each one found, it dynamically imports the `plugin.py` file,
        executes an optional `startup(di)` hook if present, and registers the
        plugin with the application's plugin registry. Errors encountered while
        loading a single plugin are logged and suppressed to allow other
        plugins to load.

        Returns:
            None:
        """
        plugins_dir = get_plugins_dir()
        plugins_dir.mkdir(parents=True, exist_ok=True)
        telemetry = self._di.resolve(Observability)
        for folder in plugins_dir.iterdir():
            if not folder.is_dir():
                continue
            path = folder / "src" / folder.name.replace("-", "_") / "plugin.py"
            if not path.exists():
                continue
            module_name = (
                folder.name.replace("-", "_")
                if folder.name.startswith("bijux_plugin_")
                else f"bijux_plugin_{folder.name.replace('-', '_')}"
            )
            try:
                plugin = load_plugin(path, module_name)
                if startup := getattr(plugin, "startup", None):
                    startup(self._di)
                self._registry.register(plugin.name, plugin, version=plugin.version)
            except Exception as e:  # pragma: no cover
                telemetry.log("error", f"Loading plugin {folder.name} failed: {e}")

    def _timeout(self) -> float:
        """Retrieves the command timeout from the configuration service.

        Returns:
            float: The command timeout duration in seconds.

        Raises:
            ValueError: If the timeout value in the configuration is malformed.
        """
        try:
            cfg = self._di.resolve(ConfigProtocol)
            raw = cfg.get("BIJUXCLI_COMMAND_TIMEOUT", default=30.0)
        except KeyError:
            raw = 30.0
        value = raw.get("value", 30.0) if isinstance(raw, dict) else raw
        try:
            return float(value)
        except (TypeError, ValueError) as err:
            raise ValueError(f"Invalid timeout configuration: {raw!r}") from err

    @property
    def di(self) -> DIContainer:
        """Read-only access to the DI container."""
        return self._di


__all__ = ["Engine"]
