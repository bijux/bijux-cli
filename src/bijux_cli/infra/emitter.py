# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Output emitter adapters."""

from __future__ import annotations

import sys
from typing import Any

import structlog

from bijux_cli.infra.serializer import serializer_for


class ConsoleEmitter:
    """Emitter that serializes and writes payloads to stdout."""

    def __init__(
        self,
        telemetry: Any,
        output_format: Any = "json",
        debug: bool = False,
        quiet: bool = False,
    ) -> None:
        self._telemetry = telemetry
        self._default_format = output_format
        self._debug = debug
        self._quiet = quiet
        self._logger = structlog.get_logger(__name__)

    def emit(
        self,
        payload: Any,
        *,
        fmt: Any | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        if self._quiet and level not in ["error", "critical"]:
            return

        output_format = fmt or self._default_format
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

        if self._debug:
            print("Diagnostics: emitted payload", file=sys.stderr)
            log = getattr(self._logger, level)
            log(message, output=stripped, **context)

        try:
            format_name = (
                str(getattr(output_format, "value"))
                if hasattr(output_format, "value")
                else str(output_format)
            )
            self._telemetry.event(
                "output_emitted", {"format": format_name, "size_chars": len(stripped)}
            )
        except Exception as tel_err:
            if self._debug:
                self._logger.error("Telemetry failed", error=str(tel_err), **context)

    def flush(self) -> None:
        """Flushes standard output."""
        sys.stdout.flush()


class NullEmitter:
    """Emitter that discards output."""

    def emit(
        self,
        payload: Any,
        *,
        fmt: Any | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        return None

    def flush(self) -> None:
        return None


__all__ = ["ConsoleEmitter", "NullEmitter"]
