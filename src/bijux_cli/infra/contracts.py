# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Infrastructure protocol contracts for Bijux CLI."""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable

from bijux_cli.core.enums import LogLevel, OutputFormat


@runtime_checkable
class Serializer(Protocol):
    """Serializer adapter for structured output."""

    def dumps(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> str:
        """Serialize data to a string."""
        ...

    def dumps_bytes(self, obj: Any, *, fmt: OutputFormat, pretty: bool) -> bytes:
        """Serialize data to bytes."""
        ...

    def loads(self, data: str | bytes, *, fmt: OutputFormat, pretty: bool) -> Any:
        """Deserialize data into a value."""
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
        fmt: OutputFormat,
        pretty: bool,
        level: LogLevel,
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
    "ProcessRunner",
    "RetryPolicy",
    "Serializer",
]
