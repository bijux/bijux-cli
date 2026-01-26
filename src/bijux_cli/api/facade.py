# SPDX-License-Identifier: Apache-2.0
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
from collections.abc import Awaitable, Callable, Iterator
from contextlib import contextmanager, suppress
import importlib
import inspect
import io
import os
from pathlib import Path
import sys
from typing import Any, cast

from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.errors import BijuxError, PluginError
from bijux_cli.core.precedence import FlagLayer, Flags, resolve_effective_config
from bijux_cli.core.runtime import run_awaitable, run_command
from bijux_cli.plugins.contracts import RegistryProtocol
from bijux_cli.services.contracts import ObservabilityProtocol, TelemetryProtocol
from bijux_cli.services.errors import ServiceError

IGNORE = {"PS1", "LS_COLORS", "PROMPT_COMMAND", "GIT_PS1_FORMAT"}
_API_GUARD_ENV = "BIJUXCLI_API_GUARD"


def _api_guard_enabled() -> bool:
    """Return True when strict API guardrails are enabled."""
    return os.environ.get(_API_GUARD_ENV) == "1"


@contextmanager
def _api_io_guard() -> Iterator[None]:
    """Ensure API calls do not write to stdout/stderr when guarded."""
    if not _api_guard_enabled():
        yield
        return
    out_buf = io.StringIO()
    err_buf = io.StringIO()
    with suppress(Exception):
        from contextlib import redirect_stderr, redirect_stdout

        with redirect_stdout(out_buf), redirect_stderr(err_buf):
            yield
    if out_buf.getvalue().strip() or err_buf.getvalue().strip():
        raise BijuxError(
            "API purity guard: stdout/stderr output is not allowed",
            http_status=500,
        )


def _consume_task(task: asyncio.Future[Any]) -> None:
    """Consumes an asyncio task to suppress unhandled exceptions."""

    def _eat_exc(t: asyncio.Future[Any]) -> None:
        """Retrieves and suppresses exceptions from a future."""
        with suppress(Exception):
            _ = t.exception()

    task.add_done_callback(_eat_exc)


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

    def __init__(self, *, log_level: LogLevel = LogLevel.INFO) -> None:
        """Initializes the `BijuxAPI` and the underlying CLI engine.

        Args:
            log_level (str): The default log level name for all underlying
                services.
        """
        DIContainer.reset()
        self._di = DIContainer.current()
        self._engine = Engine(
            self._di,
            log_level=log_level,
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
        maybe = self._tel.event(name, payload)
        if inspect.isawaitable(maybe):
            run_awaitable(cast(Awaitable[Any], maybe))

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
            exists = bool(self._await_maybe(self._registry.has(name), want_result=True))
            if exists:
                maybe = cast(Any, self._registry.deregister(name))
                self._await_maybe(maybe)
            maybe2 = cast(
                Any,
                self._registry.register(
                    name, _Wrapper(callback), alias=None, version=None
                ),
            )
            self._await_maybe(maybe2)

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
        fmt: str = "json",
        pretty: bool = True,
        log_level: LogLevel = LogLevel.INFO,
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
            fmt (str): The output format ("json" or "yaml").
            pretty (bool): If True, formats the output for readability.
            log_level (str): The requested log level.
            **kwargs (Any): Additional keyword arguments to pass to the command.

        Returns:
            Any: The result of the command's execution.
        """
        try:
            with _api_io_guard():
                return run_command(
                    self.run_async,
                    name,
                    *args,
                    quiet=quiet,
                    fmt=fmt,
                    pretty=pretty,
                    log_level=log_level,
                    **kwargs,
                )
        except SystemExit as exc:
            raise BijuxError(
                f"API purity guard: unexpected SystemExit({exc.code})",
                http_status=500,
            ) from exc

    async def run_async(
        self,
        name: str,
        *args: Any,
        quiet: bool = False,
        fmt: str = "json",
        pretty: bool = True,
        log_level: LogLevel = LogLevel.INFO,
        **kwargs: Any,
    ) -> Any:
        """Runs a command asynchronously with validation.

        This method performs validation of flags and environment variables
        before dispatching the command to the internal engine for execution.

        Args:
            name (str): The name of the command to execute.
            *args (Any): Positional arguments for the command.
            quiet (bool): If True, suppresses output.
            fmt (str): The output format ("json" or "yaml").
            pretty (bool): If True, formats the output for readability.
            log_level (str): The requested log level.
            **kwargs (Any): Additional keyword arguments to pass to the command.

        Returns:
            Any: The result of the command's execution.

        Raises:
            BijuxError: For invalid flags, unsupported formats, or internal
                execution errors.
        """
        try:
            _ = pretty
            fmt_value = OutputFormat(fmt)
            log_value = (
                log_level if isinstance(log_level, LogLevel) else LogLevel(log_level)
            )
            resolved = resolve_effective_config(
                cli=FlagLayer(
                    quiet=quiet,
                    log_level=log_value,
                    color=ColorMode.AUTO,
                    format=fmt_value,
                ),
                env=FlagLayer(),
                file=FlagLayer(),
                defaults=Flags(
                    quiet=False,
                    log_level=LogLevel.INFO,
                    color=ColorMode.AUTO,
                    format=OutputFormat.JSON,
                ),
            )
            fmt_value = resolved.flags.format

            for k, v in os.environ.items():
                if k in IGNORE:
                    continue
                if not v.isascii():
                    raise BijuxError(
                        "Non-ASCII characters in environment", http_status=400
                    )

            with _api_io_guard():
                result = await self._engine.run_command(name, *args, **kwargs)
            self._schedule_event("api.run", {"name": name})
            return result

        except PluginError as exc:
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

        except SystemExit as exc:
            raise BijuxError(
                f"API purity guard: unexpected SystemExit({exc.code})",
                http_status=500,
            ) from exc

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
        from bijux_cli.core.version import __version__
        from bijux_cli.plugins import load_plugin as _load_plugin

        p = Path(path).expanduser().resolve()
        module_name = f"bijux_plugin_{p.stem}"

        try:
            if module_name in sys.modules:
                importlib.reload(sys.modules[module_name])

            plugin = _load_plugin(p, module_name)
            plugin.startup(self._engine.di)

            exists = bool(
                self._await_maybe(self._registry.has(p.stem), want_result=True)
            )
            if exists:
                self._await_maybe(cast(Any, self._registry).deregister(p.stem))

            self._await_maybe(
                cast(Any, self._registry).register(
                    p.stem,
                    plugin,
                    alias=str(__version__),
                    version=getattr(plugin, "version", None),
                )
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

    @staticmethod
    def _await_maybe(value: Any, *, want_result: bool = False) -> Any:
        """Synchronously handle possibly-awaitable values with safe fallbacks.

        Args:
          value: A value that may or may not be awaitable (e.g., a coroutine,
            Future, Task, or a plain value).
          want_result: When `True`, and the coroutine is *scheduled* (not awaited),
            return `False` instead of `None` so callers can reliably detect that
            no immediate result is available.

        Returns:
          The original `value` if it is not awaitable; otherwise, either the
          awaited result (when run synchronously) or `None`/`False` when the
          coroutine is scheduled for background execution.

        Raises:
          Exception: Any exception raised by the coroutine when it is run
            synchronously via `asyncio.run` or `run_until_complete` is propagated.
        """
        import inspect as _inspect

        if not _inspect.isawaitable(value):
            return value

        async def _inner() -> Any:
            """Await and return the captured awaitable `value`.

            Returns:
                Any: The result produced by awaiting `value`.
            """
            return await value

        coro = _inner()

        def _close_if_possible(obj: Any) -> None:
            """Attempt to call ``close()`` on an object, suppressing errors.

            Args:
                obj: Object that may expose a callable ``close`` attribute.

            Notes:
                Any exception raised by ``close()`` is suppressed.
            """
            with suppress(Exception):
                close = getattr(obj, "close", None)
                if callable(close):
                    close()

        try:
            return run_awaitable(coro, want_result=want_result)
        finally:
            _close_if_possible(value)
            with suppress(Exception):
                coro.close()
