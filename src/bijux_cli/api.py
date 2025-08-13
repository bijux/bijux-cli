# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a high-level, synchronous facade for the Bijux CLI's core engine.

This module defines the `BijuxAPI` class, which serves as the primary public
interface for programmatic interaction with the CLI. It wraps the asynchronous
core `Engine` and other services to present a stable, thread-safe, and
synchronous API.

This facade is intended for use in integrations, testing, or any scenario
where the CLI's command and plugin management logic needs to be embedded
within another Python application.
"""

from __future__ import annotations

import asyncio
from collections.abc import Callable, Coroutine
import importlib
import os
from pathlib import Path
import sys
from typing import Any

from bijux_cli.commands.utilities import validate_common_flags
from bijux_cli.contracts import (
    ObservabilityProtocol,
    RegistryProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import BijuxError, CommandError, ServiceError

IGNORE = {"PS1", "LS_COLORS", "PROMPT_COMMAND", "GIT_PS1_FORMAT"}


class BijuxAPI:
    """A thread-safe, synchronous access layer for the Bijux CLI engine.

    This class provides a stable public API for registering commands, executing
    them, and managing plugins. It wraps the internal asynchronous `Engine` to
    allow for simpler, synchronous integration into other applications.

    Attributes:
        _di (DIContainer): The dependency injection container.
        _engine (Engine): The core asynchronous runtime engine.
        _registry (RegistryProtocol): The plugin registry service.
        _obs (ObservabilityProtocol): The logging service.
        _tel (TelemetryProtocol): The telemetry service.
    """

    def __init__(self, *, debug: bool = False) -> None:
        """Initializes the `BijuxAPI` and the underlying CLI engine.

        Args:
            debug (bool): If True, enables debug mode for all underlying
                services. Defaults to False.
        """
        DIContainer.reset()
        self._di = DIContainer.current()
        self._engine = Engine(
            self._di,
            debug=debug,
            fmt=OutputFormat.JSON,
        )
        self._registry: RegistryProtocol = self._di.resolve(RegistryProtocol)
        self._obs: ObservabilityProtocol = self._di.resolve(ObservabilityProtocol)
        self._tel: TelemetryProtocol = self._di.resolve(TelemetryProtocol)

    def _schedule_event(self, name: str, payload: dict[str, Any]) -> None:
        """Schedules a "fire-and-forget" asynchronous telemetry event.

        This helper handles the execution of async telemetry calls from a
        synchronous context.

        Args:
            name (str): The name of the telemetry event.
            payload (dict[str, Any]): The data associated with the event.
        """
        maybe_coro: Coroutine[Any, Any, None] | None = self._tel.event(name, payload)
        if asyncio.iscoroutine(maybe_coro):
            try:
                loop = asyncio.get_running_loop()
                loop.create_task(maybe_coro)
            except RuntimeError:
                asyncio.run(maybe_coro)

    def register(self, name: str, callback: Callable[..., Any]) -> None:
        """Registers or replaces a Python callable as a CLI command.

        The provided callable is wrapped to handle both synchronous and
        asynchronous functions automatically.

        Args:
            name (str): The command name to register.
            callback (Callable[..., Any]): The Python function to be executed
                when the command is run.

        Raises:
            BijuxError: If the command name is already in use or another
                registration error occurs.
        """

        class _Wrapper:
            """Wraps a user-provided callable to be executed asynchronously."""

            def __init__(self, cb: Callable[..., Any]) -> None:
                """Initializes the wrapper.

                Args:
                    cb (Callable[..., Any]): The callable to wrap.
                """
                self._cb = cb

            async def execute(self, *args: Any, **kwargs: Any) -> Any:
                """Execute the wrapped callable, awaiting if it's a coroutine.

                Args:
                    *args (Any): Positional arguments to pass to the callable.
                    **kwargs (Any): Keyword arguments to pass to the callable.

                Returns:
                    Any: The result of the callable execution.
                """
                if asyncio.iscoroutinefunction(self._cb):
                    return await self._cb(*args, **kwargs)
                return self._cb(*args, **kwargs)

        try:
            if self._registry.has(name):
                self._registry.deregister(name)
            self._registry.register(name, _Wrapper(callback))
            self._obs.log("info", "Registered command", extra={"name": name})
            self._schedule_event("api.register", {"name": name})
        except ServiceError as exc:
            self._schedule_event(
                "api.register.error", {"name": name, "error": str(exc)}
            )
            raise BijuxError(
                f"Could not register command {name}: {exc}", http_status=500
            ) from exc

    def run_sync(
        self,
        name: str,
        *args: Any,
        quiet: bool = False,
        verbose: bool = False,
        fmt: str = "json",
        pretty: bool = True,
        debug: bool = False,
        **kwargs: Any,
    ) -> Any:
        """Runs a command synchronously.

        This method is a blocking wrapper around the asynchronous `run_async`
        method. It manages the asyncio event loop to provide a simple,
        synchronous interface.

        Args:
            name (str): The name of the command to run.
            *args (Any): Positional arguments for the command.
            quiet (bool): If True, suppresses output.
            verbose (bool): If True, enables verbose logging.
            fmt (str): The output format ("json" or "yaml").
            pretty (bool): If True, formats the output for readability.
            debug (bool): If True, enables debug mode.
            **kwargs (Any): Additional keyword arguments to pass to the command.

        Returns:
            Any: The result of the command's execution.
        """
        try:
            loop = asyncio.get_running_loop()
        except RuntimeError:
            return asyncio.run(
                self.run_async(
                    name,
                    *args,
                    quiet=quiet,
                    verbose=verbose,
                    fmt=fmt,
                    pretty=pretty,
                    debug=debug,
                    **kwargs,
                )
            )
        else:
            return loop.run_until_complete(
                self.run_async(
                    name,
                    *args,
                    quiet=quiet,
                    verbose=verbose,
                    fmt=fmt,
                    pretty=pretty,
                    debug=debug,
                    **kwargs,
                )
            )

    async def run_async(
        self,
        name: str,
        *args: Any,
        quiet: bool = False,
        verbose: bool = False,
        fmt: str = "json",
        pretty: bool = True,
        debug: bool = False,
        **kwargs: Any,
    ) -> Any:
        """Runs a command asynchronously with validation.

        This method performs validation of flags and environment variables
        before dispatching the command to the internal engine for execution.

        Args:
            name (str): The name of the command to execute.
            *args (Any): Positional arguments for the command.
            quiet (bool): If True, suppresses output.
            verbose (bool): If True, enables verbose output.
            fmt (str): The output format ("json" or "yaml").
            pretty (bool): If True, formats the output for readability.
            debug (bool): If True, enables debug mode.
            **kwargs (Any): Additional keyword arguments to pass to the command.

        Returns:
            Any: The result of the command's execution.

        Raises:
            BijuxError: For invalid flags, unsupported formats, or internal
                execution errors.
        """
        try:
            fmt = fmt.lower()
            if fmt not in ("json", "yaml"):
                raise BijuxError("Unsupported format", http_status=400)

            if quiet and (verbose or debug):
                raise BijuxError(
                    "--quiet cannot be combined with --verbose/--debug", http_status=400
                )

            validate_common_flags(fmt, name, quiet, verbose or debug)

            for k, v in os.environ.items():
                if k in IGNORE:
                    continue
                if not v.isascii():
                    raise BijuxError(
                        "Non-ASCII characters in environment", http_status=400
                    )

            result = await self._engine.run_command(name, *args, **kwargs)
            self._schedule_event("api.run", {"name": name})
            return result

        except CommandError as exc:
            self._schedule_event("api.run.error", {"name": name, "error": str(exc)})
            raise BijuxError(
                f"Failed to run command {name}: {exc}", http_status=500
            ) from exc

        except ServiceError as exc:
            self._schedule_event("api.run.error", {"name": name, "error": str(exc)})
            raise BijuxError(
                f"Failed to run command {name}: {exc}", http_status=500
            ) from exc

        except BijuxError:
            raise

        except Exception as exc:
            self._schedule_event("api.run.error", {"name": name, "error": str(exc)})
            raise BijuxError(
                f"Failed to run command {name}: {exc}", http_status=500
            ) from exc

    def load_plugin(self, path: str | Path) -> None:
        """Loads or reloads a plugin module from a file path.

        This method dynamically loads the specified plugin file, initializes it,
        and registers it with the CLI system. If the plugin is already loaded,
        it is reloaded.

        Args:
            path (str | Path): The filesystem path to the plugin's Python file.

        Raises:
            BijuxError: If plugin loading, initialization, or registration fails.
        """
        from bijux_cli.__version__ import __version__
        from bijux_cli.services.plugins import load_plugin as _load_plugin

        p = Path(path).expanduser().resolve()
        module_name = f"bijux_plugin_{p.stem}"

        try:
            if module_name in sys.modules:
                importlib.reload(sys.modules[module_name])

            plugin = _load_plugin(p, module_name)
            plugin.startup(self._engine.di)

            if self._registry.has(p.stem):
                self._registry.deregister(p.stem)

            self._registry.register(
                p.stem,
                plugin,
                alias=str(__version__),
            )
            self._obs.log("info", "Loaded plugin", extra={"path": str(p)})
            self._schedule_event("api.plugin_loaded", {"path": str(p)})

        except Exception as exc:
            self._schedule_event(
                "api.plugin_load.error", {"path": str(p), "error": str(exc)}
            )
            raise BijuxError(
                f"Failed to load plugin {p}: {exc}", http_status=500
            ) from exc
