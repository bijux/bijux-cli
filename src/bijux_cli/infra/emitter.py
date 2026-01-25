# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Output emitter adapters."""

from __future__ import annotations

import sys
from typing import Any

import structlog

from bijux_cli.core.enums import LogLevel, OutputFormat
from bijux_cli.infra.serializer import serializer_for


class ConsoleEmitter:
    """Emitter that serializes and writes payloads to stdout."""

    def __init__(
        self,
        telemetry: Any,
        output_format: OutputFormat,
    ) -> None:
        """Initialize the console emitter."""
        self._telemetry = telemetry
        self._output_format = output_format
        self._logger = structlog.get_logger(__name__)

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat,
        pretty: bool = False,
        level: LogLevel = LogLevel.INFO,
        message: str = "Emitting output",
        output: str | None = None,
        emit_output: bool = True,
        emit_diagnostics: bool = False,
        **context: Any,
    ) -> None:
        """Serialize and emit a payload."""
        if not emit_output:
            return

        output_format = fmt
        serializer = serializer_for(output_format, self._telemetry)
        try:
            output_str = serializer.dumps(payload, fmt=output_format, pretty=pretty)
        except Exception as error:
            self._logger.error("Serialization failed", error=str(error), **context)
            raise RuntimeError(f"Serialization failed: {error}") from error

        stripped = output_str.rstrip("\n")
        if output:
            with open(output, "w", encoding="utf-8") as f:
                f.write(stripped)
        else:
            print(stripped, file=sys.stdout, flush=True)

        try:
            format_name = output_format.value
            self._telemetry.event(
                "output_emitted",
                {"format": format_name, "size_chars": len(stripped)},
            )
        except Exception as tel_err:
            self._logger.debug("Telemetry failed", error=str(tel_err), **context)

    def flush(self) -> None:
        """Flushes standard output."""
        sys.stdout.flush()


class NullEmitter:
    """Emitter that discards output."""

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat,
        pretty: bool = False,
        level: LogLevel = LogLevel.INFO,
        message: str = "Emitting output",
        output: str | None = None,
        emit_output: bool = True,
        emit_diagnostics: bool = False,
        **context: Any,
    ) -> None:
        """Drop emitted payloads."""
        _ = (
            payload,
            fmt,
            pretty,
            level,
            message,
            output,
            emit_output,
            emit_diagnostics,
        )
        _ = context
        return None

    def flush(self) -> None:
        """No-op flush for the null emitter."""
        return None


__all__ = ["ConsoleEmitter", "NullEmitter"]
