# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the concrete implementation of the structured output emitter service.

This module defines the `Emitter` class, which implements the `EmitterProtocol`.
It is responsible for serializing data payloads into structured formats like
JSON or YAML and writing them to standard output or a specified file. The
service also integrates with telemetry to log emission events.
"""

from __future__ import annotations

import sys
from typing import Any

from injector import inject
import structlog

from bijux_cli.contracts import EmitterProtocol, TelemetryProtocol
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import CommandError
from bijux_cli.infra.serializer import serializer_for


class Emitter(EmitterProtocol):  # pyright: ignore[reportUntypedBaseClass]
    """A service for serializing and emitting structured output.

    This class implements the `EmitterProtocol`. It handles the serialization
    of data payloads and writes the result to standard output or a file, while
    also tracking events with a telemetry service.

    Attributes:
        _telemetry (TelemetryProtocol): The telemetry service for event tracking.
        _default_format (OutputFormat): The default format for serialization.
        _debug (bool): Flag indicating if debug mode is enabled.
        _quiet (bool): Flag indicating if normal output should be suppressed.
        _logger: A configured `structlog` logger instance.
    """

    @inject
    def __init__(
        self,
        telemetry: TelemetryProtocol,
        output_format: OutputFormat = OutputFormat.JSON,
        debug: bool = False,
        quiet: bool = False,
        **kwargs: Any,
    ):
        """Initializes the Emitter service.

        Args:
            telemetry (TelemetryProtocol): The telemetry service for event tracking.
            output_format (OutputFormat): The default output format for emissions.
            debug (bool): If True, enables debug logging.
            quiet (bool): If True, suppresses all non-error output.
            **kwargs: Additional keyword arguments (unused).
        """
        self._telemetry = telemetry
        self._default_format = output_format
        self._debug = debug
        self._quiet = quiet
        self._logger = structlog.get_logger(__name__)

    def emit(
        self,
        payload: Any,
        *,
        fmt: OutputFormat | None = None,
        pretty: bool = False,
        level: str = "info",
        message: str = "Emitting output",
        output: str | None = None,
        **context: Any,
    ) -> None:
        """Serializes and emits a structured data payload.

        The payload is serialized to the specified format and written to stdout
        or a file path if provided. The operation is suppressed if the emitter
        is in quiet mode and the log level is not critical.

        Args:
            payload (Any): The data payload to serialize and emit.
            fmt (OutputFormat | None): The output format. If None, the service's
                default format is used.
            pretty (bool): If True, formats the output for human readability.
            level (str): The log level for any accompanying message (e.g.,
                "info", "debug", "error").
            message (str): A descriptive message for logging purposes.
            output (str | None): An optional file path to write the output to.
                If None, output is written to `sys.stdout`.
            **context (Any): Additional key-value pairs for structured logging.

        Returns:
            None:

        Raises:
            CommandError: If the payload cannot be serialized.
        """
        if self._quiet and level not in ["error", "critical"]:
            return

        output_format = fmt or self._default_format
        serializer = serializer_for(output_format, self._telemetry)
        try:
            output_str = serializer.dumps(payload, fmt=output_format, pretty=pretty)
        except Exception as error:
            self._logger.error("Serialization failed", error=str(error), **context)
            raise CommandError(
                f"Serialization failed: {error}", http_status=500
            ) from error

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
            self._telemetry.event(
                "output_emitted",
                {"format": output_format.value, "size_chars": len(stripped)},
            )
        except Exception as tel_err:
            if self._debug:
                self._logger.error("Telemetry failed", error=str(tel_err), **context)

    def flush(self) -> None:
        """Flushes any buffered output to standard output."""
        sys.stdout.flush()


__all__ = ["Emitter"]
