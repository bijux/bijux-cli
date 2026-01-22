# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Core protocol contracts for Bijux CLI."""

from __future__ import annotations

from typing import Any, Protocol, Self, runtime_checkable


@runtime_checkable
class ExecutionContext(Protocol):
    """Execution-scoped context carrier."""

    def set(self, key: str, value: Any) -> None:
        """Store a value in the context."""
        ...

    def get(self, key: str) -> Any:
        """Retrieve a value from the context."""
        ...

    def clear(self) -> None:
        """Clear all context data."""
        ...

    def __enter__(self) -> Self:
        """Enter the synchronous context manager."""
        ...

    def __exit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exit the synchronous context manager."""
        ...

    async def __aenter__(self) -> Self:
        """Enter the async context manager."""
        ...

    async def __aexit__(self, _exc_type: Any, _exc_value: Any, traceback: Any) -> None:
        """Exit the async context manager."""
        ...


@runtime_checkable
class Serializer(Protocol):
    """Serializer adapter for structured output."""

    def dumps(self, obj: Any, *, fmt: str, pretty: bool) -> str:
        """Serialize data to a string."""
        ...

    def dumps_bytes(self, obj: Any, *, fmt: str, pretty: bool) -> bytes:
        """Serialize data to bytes."""
        ...

    def loads(self, data: str | bytes, *, fmt: str, pretty: bool) -> Any:
        """Deserialize data into a value."""
        ...

    def emit(self, payload: Any, *, fmt: str, pretty: bool) -> None:
        """Serialize and emit a payload."""
        ...


@runtime_checkable
class RetryPolicy(Protocol):
    """Retry policy for transient failures."""

    def run(self, fn: Any, *args: Any, **kwargs: Any) -> Any:
        """Execute a callable with retry behavior."""
        ...

    def reset(self) -> None:
        """Reset any internal retry state."""
        ...


@runtime_checkable
class Emitter(Protocol):
    """Emitter for structured output."""

    def emit(
        self,
        payload: Any,
        *,
        fmt: str | None,
        pretty: bool,
        level: str,
        message: str,
        output: str | None,
        emit_output: bool = True,
        emit_diagnostics: bool = False,
        **context: Any,
    ) -> None:
        """Serialize and emit a structured payload."""
        ...

    def flush(self) -> None:
        """Flush any buffered output."""
        ...


@runtime_checkable
class ProcessRunner(Protocol):
    """Runner for isolated command execution."""

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Run a command with an executor."""
        ...

    def shutdown(self) -> None:
        """Shut down the runner."""
        ...

    def get_status(self) -> dict[str, Any]:
        """Return runner status info."""
        ...


__all__ = [
    "Emitter",
    "ExecutionContext",
    "ProcessRunner",
    "RetryPolicy",
    "Serializer",
]
