# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Provides shared utilities for the `bijux memory` command group.

This module centralizes common logic used by the memory-related subcommands.
Its primary responsibility is to provide a consistent way to resolve the
`MemoryProtocol` service from the Dependency Injection (DI) container,
including standardized error handling for cases where the service is
unavailable.
"""

from __future__ import annotations

from bijux_cli.cli.core.command import raise_exit_intent
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.services.diagnostics.contracts import MemoryProtocol


def resolve_memory_service(
    command: str,
    fmt_lower: OutputFormat,
    quiet: bool,
    include_runtime: bool,
    log_level: LogLevel,
) -> MemoryProtocol:
    """Resolves the MemoryProtocol implementation from the DI container.

    Args:
        command (str): The full command name (e.g., "memory list").
        fmt_lower (OutputFormat): The chosen output format.
        quiet (bool): If True, suppresses non-error output.
        include_runtime (bool): If True, includes runtime metadata in errors.
        log_level (LogLevel): Logging level for diagnostics.

    Returns:
        MemoryProtocol: An instance of the memory service.

    Raises:
        SystemExit: Exits with a structured error if the service cannot be
            resolved from the container.
    """
    try:
        return DIContainer.current().resolve(MemoryProtocol)
    except Exception as exc:
        raise_exit_intent(
            f"Memory service unavailable: {exc}",
            code=1,
            failure="service_unavailable",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
        )
