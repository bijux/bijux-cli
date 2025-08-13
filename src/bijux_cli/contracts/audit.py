# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the audit logging and diagnostics service.

This module specifies the `AuditProtocol`, a formal interface that any
auditing service within the application must implement. This ensures a
consistent API for logging command executions and performing system health
checks, promoting modularity and testability.
"""

from __future__ import annotations

from typing import Any, Protocol, TypeVar, runtime_checkable

from bijux_cli.contracts.process import ProcessPoolProtocol

T = TypeVar("T")


@runtime_checkable
class AuditProtocol(
    ProcessPoolProtocol,
    Protocol,  # pyright: ignore[reportUntypedBaseClass]
):
    """Defines the contract for audit logging and system diagnostics.

    This interface specifies the methods required for securely logging command
    executions, retrieving audit trails, and performing diagnostic checks on
    the CLI environment. It inherits from `ProcessPoolProtocol` to manage
    command execution.
    """

    def log(self, cmd: list[str], *, executor: str) -> None:
        """Logs a command execution for auditing purposes.

        Args:
            cmd (list[str]): The command and its arguments to log.
            executor (str): The name or identifier of the entity that executed
                the command.

        Returns:
            None:
        """
        ...

    def get_commands(self) -> list[dict[str, Any]]:
        """Returns a copy of all recorded audit commands.

        Returns:
            list[dict[str, Any]]: A list of dictionaries, where each dictionary
                represents a logged command execution.
        """
        ...

    def cli_audit(self) -> None:
        """Performs a CLI-specific audit and status check.

        This method is the entry point for the `bijux audit` command.
        Implementations should gather and log the current audit status without
        raising exceptions or executing external commands.

        Returns:
            None:
        """
        ...
