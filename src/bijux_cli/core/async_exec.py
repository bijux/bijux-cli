# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Async execution rules and adapters for CLI commands.

Single source of truth:
- The CLI owns the event loop and starts it via `anyio.run`.
- Async commands are awaited directly on that loop.
- Sync commands are executed via `anyio.to_thread.run_sync` to avoid blocking.

Do not call `asyncio.run()` outside this module.
"""

from __future__ import annotations

from collections.abc import Awaitable, Callable
import asyncio
import contextlib
import functools
import inspect
from typing import Any, TypeVar

import anyio
import typer

T = TypeVar("T")


async def _execute(func: Callable[..., Any], *args: Any, **kwargs: Any) -> Any:
    if inspect.iscoroutinefunction(func):
        return await func(*args, **kwargs)
    return await anyio.to_thread.run_sync(functools.partial(func, *args, **kwargs))


def run_command(func: Callable[..., Any], *args: Any, **kwargs: Any) -> Any:
    """Run a callable under the CLI-owned event loop."""
    async def _inner() -> Any:
        return await _execute(func, *args, **kwargs)

    return anyio.run(_inner)


def run_awaitable(value: Awaitable[T], *, want_result: bool = False) -> T | bool | None:
    """Synchronously handle an awaitable, scheduling if already in a loop."""
    async def _inner() -> T:
        return await value

    try:
        loop = asyncio.get_running_loop()
    except RuntimeError:
        return anyio.run(_inner)

    if hasattr(loop, "create_task"):
        loop.create_task(_inner())
        return False if want_result else None

    run_uc = getattr(loop, "run_until_complete", None)
    if callable(run_uc):
        result = run_uc(_inner())
        close = getattr(value, "close", None)
        if callable(close):
            with contextlib.suppress(Exception):
                close()
        return result

    close = getattr(value, "close", None)
    if callable(close):
        close()

    return False if want_result else None


def command_adapter(func: Callable[..., Any]) -> Callable[..., Any]:
    """Wrap a command function so all execution flows through the adapter."""
    if getattr(func, "_bijux_async_adapter", False):
        return func

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        return run_command(func, *args, **kwargs)

    wrapper._bijux_async_adapter = True  # type: ignore[attr-defined]
    return wrapper


def adapt_typer(app: typer.Typer) -> None:
    """Ensure a Typer app's callbacks/commands are routed via the adapter."""
    callback = getattr(app, "registered_callback", None)
    if callback and getattr(callback, "callback", None):
        callback.callback = command_adapter(callback.callback)

    for cmd in getattr(app, "registered_commands", []) or []:
        if getattr(cmd, "callback", None):
            cmd.callback = command_adapter(cmd.callback)

    for grp in getattr(app, "registered_groups", []) or []:
        if getattr(grp, "typer_instance", None):
            adapt_typer(grp.typer_instance)


class AsyncTyper(typer.Typer):
    """Typer subclass that routes all commands through the async adapter."""

    def command(self, *args: Any, **kwargs: Any) -> Callable[[Callable[..., Any]], Any]:
        decorator = super().command(*args, **kwargs)

        def wrapper(func: Callable[..., Any]) -> Any:
            return decorator(command_adapter(func))

        return wrapper

    def callback(self, *args: Any, **kwargs: Any) -> Callable[[Callable[..., Any]], Any]:
        decorator = super().callback(*args, **kwargs)

        def wrapper(func: Callable[..., Any]) -> Any:
            return decorator(command_adapter(func))

        return wrapper

    def add_typer(self, *args: Any, **kwargs: Any) -> None:
        super().add_typer(*args, **kwargs)
        if args:
            sub = args[0]
            if isinstance(sub, typer.Typer):
                adapt_typer(sub)
