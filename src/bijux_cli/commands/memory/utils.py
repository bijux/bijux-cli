# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides shared utilities for the `bijux memory` command group.

This module centralizes common logic used by the memory-related subcommands.
Its primary responsibility is to provide a consistent way to resolve the
`MemoryProtocol` service from the Dependency Injection (DI) container,
including standardized error handling for cases where the service is
unavailable.
"""

from __future__ import annotations

from bijux_cli.commands.utilities import emit_error_and_exit
from bijux_cli.contracts import MemoryProtocol
from bijux_cli.core.di import DIContainer


def resolve_memory_service(
    command: str,
    fmt_lower: str,
    quiet: bool,
    include_runtime: bool,
    debug: bool,
) -> MemoryProtocol:
    """Resolves the MemoryProtocol implementation from the DI container.

    Args:
        command (str): The full command name (e.g., "memory list").
        fmt_lower (str): The chosen output format, lowercased.
        quiet (bool): If True, suppresses non-error output.
        include_runtime (bool): If True, includes runtime metadata in errors.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        MemoryProtocol: An instance of the memory service.

    Raises:
        SystemExit: Exits with a structured error if the service cannot be
            resolved from the container.
    """
    try:
        return DIContainer.current().resolve(MemoryProtocol)
    except Exception as exc:
        emit_error_and_exit(
            f"Memory service unavailable: {exc}",
            code=1,
            failure="service_unavailable",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
        )
