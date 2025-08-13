# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a concrete implementation for request-scoped context management.

This module defines the `Context` class, which implements the `ContextProtocol`.
It uses Python's `contextvars` to provide a thread-safe and async-safe
mechanism for storing and retrieving key-value data associated with a specific
command execution or request. This allows state to be passed through the
application's call stack without explicit argument passing.
"""

from __future__ import annotations

from collections.abc import Iterator
from contextlib import contextmanager
from contextvars import ContextVar, Token
import os
from typing import Any, TypeVar

from injector import inject

from bijux_cli.contracts import ContextProtocol, ObservabilityProtocol
from bijux_cli.core.di import DIContainer

T = TypeVar("T")
_current_context: ContextVar[dict[str, Any] | None] = ContextVar(
    "current_context", default=None
)


class Context(ContextProtocol):
    """Provides thread-safe, request-scoped storage for CLI commands.

    This class uses `contextvars` to manage a dictionary of data that is
    isolated to the current task or thread. It is intended to be used as
    both a synchronous and asynchronous context manager.

    Attributes:
        _di (DIContainer): The dependency injection container.
        _log (ObservabilityProtocol): The logging service.
        _data (dict[str, Any]): The dictionary storing the context's data.
        _token (Token | None): The token for resetting the `ContextVar`.
    """

    @inject
    def __init__(self, di: DIContainer) -> None:
        """Initializes a new Context instance.

        Args:
            di (DIContainer): The dependency injection container used to
                resolve the logging service.
        """
        self._di = di
        self._log: ObservabilityProtocol = di.resolve(ObservabilityProtocol)
        self._data: dict[str, Any] = {}
        self._token: Token[dict[str, Any] | None] | None = None
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log("debug", "Context initialized", extra={})

    def set(self, key: str, value: Any) -> None:
        """Sets a value in the current context's data.

        Args:
            key (str): The key for the value.
            value (Any): The value to store.

        Returns:
            None:
        """
        self._data[key] = value
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log(
                "debug", "Context set", extra={"key": key, "value": str(value)}
            )

    def get(self, key: str) -> Any:
        """Gets a value from the current context's data.

        Args:
            key (str): The key of the value to retrieve.

        Returns:
            Any: The value associated with the key.

        Raises:
            KeyError: If the key is not found in the context.
        """
        if key not in self._data:
            if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
                self._log.log("warning", "Context key not found", extra={"key": key})
            raise KeyError(f"Key '{key}' not found in context")
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log(
                "debug",
                "Context get",
                extra={"key": key, "value": str(self._data[key])},
            )
        return self._data[key]

    def clear(self) -> None:
        """Removes all values from the context's data."""
        self._data.clear()
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log("debug", "Context cleared", extra={})

    def __enter__(self) -> Context:
        """Enters the context as a synchronous manager.

        This sets the current `contextvar` to this instance's data dictionary.

        Returns:
            Context: The context instance itself.
        """
        self._token = _current_context.set(self._data)
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log("debug", "Context entered", extra={})
        return self

    def __exit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exits the synchronous context manager.

        This resets the `contextvar` to its previous state.

        Args:
            _exc_type (Any): Exception type, if any (unused).
            _exc_value (Any): Exception value, if any (unused).
            traceback (Any): Traceback, if any (unused).

        Returns:
            None:
        """
        if self._token:
            _current_context.reset(self._token)
            self._token = None
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log("debug", "Context exited", extra={})

    async def __aenter__(self) -> Context:
        """Enters the context as an asynchronous manager.

        This sets the current `contextvar` to this instance's data dictionary.

        Returns:
            Context: The context instance itself.
        """
        self._token = _current_context.set(self._data)
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log("debug", "Async context entered", extra={})
        return self

    async def __aexit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exits the asynchronous context manager.

        This resets the `contextvar` to its previous state.

        Args:
            _exc_type (Any): Exception type, if any (unused).
            _exc_value (Any): Exception value, if any (unused).
            traceback (Any): Traceback, if any (unused).

        Returns:
            None:
        """
        if self._token:
            _current_context.reset(self._token)
            self._token = None
        if os.getenv("VERBOSE_DI") and not os.getenv("BIJUXCLI_TEST_MODE"):
            self._log.log("debug", "Async context exited", extra={})

    @classmethod
    def current_data(cls) -> dict[str, Any]:
        """Returns the dictionary for the currently active CLI context.

        This provides direct access to the data stored in the underlying
        `contextvar` for the current execution scope.

        Returns:
            dict[str, Any]: The active context data dictionary.
        """
        data = _current_context.get()
        if data is None:
            data = {}
            _current_context.set(data)
        return data

    @classmethod
    def set_current_data(cls, data: dict[str, Any]) -> None:
        """Sets the dictionary for the currently active CLI context.

        Args:
            data (dict[str, Any]): The data to use for the active context.

        Returns:
            None:
        """
        _current_context.set(data)

    @classmethod
    @contextmanager
    def use_context(cls, data: dict[str, Any]) -> Iterator[None]:
        """Temporarily replaces the current context data within a `with` block.

        Args:
            data (dict[str, Any]): The dictionary to use as the context for
                the duration of the `with` block.

        Yields:
            None:
        """
        token = _current_context.set(data)
        try:
            yield
        finally:
            _current_context.reset(token)


__all__ = ["Context"]
