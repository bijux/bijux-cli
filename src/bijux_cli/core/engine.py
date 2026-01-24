# SPDX-License-Identifier: Apache-2.0
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
import os
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from bijux_cli.core.di import DIContainer

from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.errors import PluginError
from bijux_cli.core.precedence import resolve_output_flags
from bijux_cli.plugins import get_plugins_dir, load_plugin
from bijux_cli.plugins.contracts import RegistryProtocol
from bijux_cli.plugins.services import register_plugin_services
from bijux_cli.services import register_default_services
from bijux_cli.services.history import History
from bijux_cli.services.logging.contracts import LoggingConfig
from bijux_cli.services.logging.observability import Observability


class Engine:
    """Orchestrates the CLI's runtime services and plugin lifecycle.

    Attributes:
        _di (DIContainer): The dependency injection container.
        _format (OutputFormat): The default output format.
        _quiet (bool): The quiet mode flag.
        _registry (RegistryProtocol): The plugin registry service.
    """

    def __init__(
        self,
        di: Any = None,
        *,
        log_level: LogLevel = LogLevel.INFO,
        fmt: OutputFormat = OutputFormat.JSON,
        quiet: bool = False,
        logging_config: LoggingConfig | None = None,
    ) -> None:
        """Initializes the engine and its core services.

        This sets up the DI container, registers default services, and loads
        all discoverable plugins.

        Args:
            di (Any, optional): An existing dependency injection container. If
                None, the global singleton instance is used. Defaults to None.
            log_level (LogLevel): The default log level for services.
            fmt (OutputFormat): The default output format for services.
            quiet (bool): If True, suppresses output from services.
            logging_config (LoggingConfig | None): Optional logging configuration
                override for service registration.
        """
        from bijux_cli.core.di import DIContainer

        self._di = di or DIContainer.current()
        self._format = fmt
        self._quiet = quiet
        if logging_config is None:
            resolved = resolve_output_flags(
                quiet=quiet,
                pretty=True,
                log_level=log_level,
                color=ColorMode.AUTO,
            )
            logging_config = LoggingConfig(
                quiet=quiet,
                log_level=resolved.log_level,
                color=resolved.color,
            )
        register_default_services(
            self._di,
            logging_config=logging_config,
            output_format=fmt,
        )
        register_plugin_services(self._di)
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
            PluginError: If the plugin is not found, its `execute` method
                is invalid, or if it fails during execution.
        """
        plugin = self._registry.get(name)
        execute = getattr(plugin, "execute", None)
        if not callable(execute):
            raise PluginError(
                f"Plugin '{name}' has no callable 'execute' method.", http_status=404
            )
        if not inspect.iscoroutinefunction(execute):
            raise PluginError(
                f"Plugin '{name}' 'execute' is not async/coroutine.", http_status=400
            )
        try:
            return await asyncio.wait_for(execute(*args, **kwargs), self._timeout())
        except Exception as exc:  # pragma: no cover
            raise PluginError(f"Failed to run plugin '{name}': {exc}") from exc

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
                self._registry.register(
                    plugin.name, plugin, alias=None, version=plugin.version
                )
            except Exception as e:  # pragma: no cover
                telemetry.log("error", f"Loading plugin {folder.name} failed: {e}")

    def _timeout(self) -> float:
        """Retrieves the command timeout from the configuration service.

        Returns:
            float: The command timeout duration in seconds.

        Raises:
            ValueError: If the timeout value in the configuration is malformed.
        """
        from bijux_cli.cli.core.constants import ENV_COMMAND_TIMEOUT

        raw = os.getenv(ENV_COMMAND_TIMEOUT, "30.0")
        try:
            return float(raw)
        except (TypeError, ValueError) as err:
            raise ValueError(f"Invalid timeout configuration: {raw!r}") from err

    @property
    def di(self) -> DIContainer:
        """Read-only access to the DI container."""
        return self._di


__all__ = ["Engine"]
