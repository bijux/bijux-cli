# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides concrete audit service implementations.

This module defines concrete classes that implement the `AuditProtocol`. It
offers different strategies for handling command auditing and execution,
allowing the application to switch between a simulation mode (`DryRunAudit`)
and a real execution mode (`RealAudit`).

A factory function, `get_audit_service`, is provided to select the
appropriate implementation based on a `dry_run` flag.
"""

from __future__ import annotations

from contextlib import suppress
import subprocess  # nosec B404
from typing import Any

from injector import inject

from bijux_cli.contracts import AuditProtocol, ObservabilityProtocol, TelemetryProtocol
from bijux_cli.core.exceptions import BijuxError
from bijux_cli.services.utils import validate_command


class _BaseAudit(AuditProtocol):
    """A base class providing common logic for audit services.

    Attributes:
        _log (ObservabilityProtocol): The logging service.
        _tel (TelemetryProtocol): The telemetry service for event tracking.
        _commands (list): An in-memory list of logged command events.
    """

    _log: ObservabilityProtocol
    _tel: TelemetryProtocol
    _commands: list[dict[str, Any]]

    @inject
    def __init__(self, log: ObservabilityProtocol, tel: TelemetryProtocol) -> None:
        """Initializes the base audit service.

        Args:
            log (ObservabilityProtocol): The service for structured logging.
            tel (TelemetryProtocol): The service for event tracking.
        """
        self._log: ObservabilityProtocol = log
        self._tel: TelemetryProtocol = tel
        self._commands: list[dict[str, Any]] = []

    def shutdown(self) -> None:
        """Flushes telemetry and closes the logger, suppressing errors."""
        with suppress(Exception):
            self._tel.flush()
        with suppress(Exception):
            self._log.close()

    def log(self, cmd: list[str], *, executor: str) -> None:
        """Logs a command execution for auditing purposes.

        Args:
            cmd (list[str]): The command and its arguments to log.
            executor (str): The name of the entity executing the command.

        Returns:
            None:
        """
        pass

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Validates, logs, and executes the given command securely.

        Args:
            cmd (list[str]): The command and its arguments to execute.
            executor (str): The name of the entity executing the command.

        Returns:
            tuple[int, bytes, bytes]: A tuple containing the command's return
                code, standard output, and standard error.

        Raises:
            NotImplementedError: This method must be implemented by subclasses.
        """
        raise NotImplementedError("Subclasses must implement 'run' method.")

    def get_commands(self) -> list[dict[str, Any]]:
        """Returns a copy of the recorded command audit trail.

        Returns:
            list[dict[str, Any]]: A list of dictionaries, where each represents
                a logged command.
        """
        return self._commands.copy()

    def get_status(self) -> dict[str, Any]:
        """Returns the current status of the audit service.

        Returns:
            dict[str, Any]: A dictionary containing status information, such as
                the number of commands processed.
        """
        return {"commands_processed": len(self._commands)}

    def cli_audit(self) -> None:
        """Performs a no-op CLI audit to conform to the protocol."""
        pass


class DryRunAudit(_BaseAudit):
    """An audit service that records events and simulates command execution."""

    def __init__(self, log: ObservabilityProtocol, tel: TelemetryProtocol) -> None:
        """Initializes the `DryRunAudit` service.

        Args:
            log (ObservabilityProtocol): The service for structured logging.
            tel (TelemetryProtocol): The service for event tracking.
        """
        super().__init__(log, tel)

    def log(self, cmd: list[str], *, executor: str) -> None:
        """Logs and records a command without executing it.

        Args:
            cmd (list[str]): The command and arguments to log.
            executor (str): The name of the entity executing the command.

        Returns:
            None:
        """
        entry = {"cmd": cmd, "executor": executor}
        self._commands.append(entry)
        self._log.log("info", "Dry-run", extra=entry)
        self._tel.event("audit_dry_run", entry)

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Simulates the execution of a command.

        This method logs the command and returns a successful result without
        actually running a subprocess.

        Args:
            cmd (list[str]): The command to simulate.
            executor (str): The name of the entity executing the command.

        Returns:
            tuple[int, bytes, bytes]: A tuple of dummy values: `(0, b"", b"")`.
        """
        self.log(cmd, executor=executor)
        return 0, b"", b""

    def cli_audit(self) -> None:
        """Logs a dry-run CLI audit event."""
        self._log.log("info", "CLI audit (dry-run)", extra={})
        self._tel.event("audit_cli_dry_run", {})


class RealAudit(_BaseAudit):
    """An audit service that validates, logs, and executes real commands."""

    def __init__(self, log: ObservabilityProtocol, tel: TelemetryProtocol) -> None:
        """Initializes the `RealAudit` service.

        Args:
            log (ObservabilityProtocol): The service for structured logging.
            tel (TelemetryProtocol): The service for event tracking.
        """
        super().__init__(log, tel)

    def log(self, cmd: list[str], *, executor: str) -> None:
        """Logs a command with the intent to execute it.

        Args:
            cmd (list[str]): The command and arguments to log.
            executor (str): The name of the entity executing the command.

        Returns:
            None:
        """
        entry = {"cmd": cmd, "executor": executor}
        self._commands.append(entry)
        self._log.log("debug", f"Executing {executor}", extra=entry)
        self._tel.event("audit_execute", entry)

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Validates, logs, and executes a command in a subprocess.

        Args:
            cmd (list[str]): The command and arguments to execute.
            executor (str): The name of the entity executing the command.

        Returns:
            tuple[int, bytes, bytes]: A tuple containing the command's return
                code, standard output, and standard error.

        Raises:
            BijuxError: If command validation fails or an unexpected error
                occurs during execution.
        """
        try:
            safe_cmd = validate_command(cmd)
            self.log(safe_cmd, executor=executor)
            proc = subprocess.run(  # noqa: S603 # nosec B603
                safe_cmd,
                capture_output=True,
                check=False,
                shell=False,
            )
            self._tel.event(
                "audit_executed",
                {
                    "cmd": safe_cmd,
                    "executor": executor,
                    "returncode": proc.returncode,
                },
            )
            return proc.returncode, proc.stdout, proc.stderr
        except BijuxError as err:
            self._tel.event(
                "audit_execution_failed",
                {"cmd": cmd, "executor": executor, "error": str(err)},
            )
            raise
        except Exception as err:
            self._tel.event(
                "audit_execution_failed",
                {"cmd": cmd, "executor": executor, "error": str(err)},
            )
            raise BijuxError(f"Failed to execute {executor!r}: {err}") from err

    def cli_audit(self) -> None:
        """Logs a real CLI audit event."""
        self._log.log(
            "info", "CLI audit (real)", extra={"commands": len(self._commands)}
        )
        self._tel.event("audit_cli_real", {"commands": len(self._commands)})


def get_audit_service(
    observability: ObservabilityProtocol,
    telemetry: TelemetryProtocol,
    dry_run: bool = False,
) -> AuditProtocol:
    """A factory function for creating an audit service instance.

    Args:
        observability (ObservabilityProtocol): The service for logging.
        telemetry (TelemetryProtocol): The service for event tracking.
        dry_run (bool): If True, returns a `DryRunAudit` instance; otherwise,
            returns a `RealAudit` instance.

    Returns:
        AuditProtocol: An instance of the appropriate audit service.
    """
    return (
        DryRunAudit(observability, telemetry)
        if dry_run
        else RealAudit(observability, telemetry)
    )


__all__ = [
    "DryRunAudit",
    "RealAudit",
    "get_audit_service",
]
