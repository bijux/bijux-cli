# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides a process pool service for executing external commands.

This module defines the `ProcessPool` class, a concrete implementation of the
`ProcessPoolProtocol`. It is designed to run shell commands in isolated
subprocesses using a managed pool of workers. Key features include command
validation to prevent shell injection and an in-memory LRU cache to return
results for repeated commands without re-execution.
"""

from __future__ import annotations

from collections import OrderedDict
from concurrent.futures import ProcessPoolExecutor
import os
import shutil
import subprocess  # nosec B404
from typing import Any

from injector import inject

from bijux_cli.core.contracts import (
    ObservabilityProtocol,
    ProcessPoolProtocol,
    TelemetryProtocol,
)
from bijux_cli.core.errors import BijuxError


def validate_command(cmd: list[str]) -> list[str]:
    """Validates a command and its arguments against a whitelist."""
    if not cmd:
        raise BijuxError("Empty command not allowed", http_status=403)
    env_val = os.getenv("BIJUXCLI_ALLOWED_COMMANDS")
    allowed_commands = (env_val or "echo,ls,cat,grep").split(",")

    cmd_name = os.path.basename(cmd[0])
    if cmd_name not in allowed_commands:
        raise BijuxError(
            f"invalid command {cmd_name!r} not in allowed list: {allowed_commands}",
            http_status=403,
        )
    resolved_cmd_path = shutil.which(cmd[0])
    if not resolved_cmd_path:
        raise BijuxError(
            f"Command not found or not executable: {cmd[0]!r}", http_status=403
        )
    if os.path.basename(resolved_cmd_path) != cmd_name:
        raise BijuxError(f"Disallowed command path: {cmd[0]!r}", http_status=403)
    cmd[0] = resolved_cmd_path
    forbidden = set(";|&><`!")
    for arg in cmd[1:]:
        if any(ch in arg for ch in forbidden):
            raise BijuxError(f"Unsafe argument: {arg!r}", http_status=403)
    return cmd


class ProcessPool(ProcessPoolProtocol):
    """Executes validated commands in a worker pool with an LRU cache.

    This class manages a `ProcessPoolExecutor` to run commands in separate
    processes. It maintains a cache of recent command results to avoid
    unnecessary re-execution.

    Attributes:
        _MAX_CACHE (int): The maximum number of command results to store in the
            LRU cache.
        _exec (ProcessPoolExecutor): The underlying executor for running tasks.
        _log (ObservabilityProtocol): The logging service.
        _tel (TelemetryProtocol): The telemetry service.
        _cache (OrderedDict): An LRU cache storing tuples of command arguments
            to their results `(returncode, stdout, stderr)`.
    """

    _MAX_CACHE = 1000

    @inject
    def __init__(
        self,
        observability: ObservabilityProtocol,
        telemetry: TelemetryProtocol,
        max_workers: int = 4,
    ) -> None:
        """Initializes the `ProcessPool` service.

        Args:
            observability (ObservabilityProtocol): The service for logging.
            telemetry (TelemetryProtocol): The service for event tracking.
            max_workers (int): The maximum number of worker processes. This can
                be overridden by the `BIJUXCLI_MAX_WORKERS` environment variable.
        """
        max_workers = int(os.getenv("BIJUXCLI_MAX_WORKERS", str(max_workers)))
        self._exec = ProcessPoolExecutor(max_workers=max_workers)
        self._log = observability
        self._tel = telemetry
        self._cache: OrderedDict[tuple[str, ...], tuple[int, bytes, bytes]] = (
            OrderedDict()
        )

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Executes a command in the process pool, using a cache.

        The command is first looked up in the LRU cache. If not found, it is
        validated and then executed in a subprocess. The result is then stored
        in the cache.

        Args:
            cmd (list[str]): The command and its arguments to execute.
            executor (str): The name of the entity requesting the execution,
                used for telemetry.

        Returns:
            tuple[int, bytes, bytes]: A tuple containing the command's return
                code, standard output, and standard error.

        Raises:
            BijuxError: If command validation fails or if an unexpected error
                occurs during subprocess execution.
        """
        key = tuple(cmd)
        if key in self._cache:
            self._log.log("debug", "Process-pool cache hit", extra={"cmd": cmd})
            self._tel.event("procpool_cache_hit", {"cmd": cmd, "executor": executor})
            self._cache.move_to_end(key)
            return self._cache[key]

        try:
            orig_cmd = list(cmd)
            validate = __import__(
                "bijux_cli.services.diagnostics.process", fromlist=["validate_command"]
            ).validate_command
            safe_cmd = validate(cmd)
            self._log.log("info", "Process-pool executing", extra={"cmd": orig_cmd})
            self._tel.event("procpool_execute", {"cmd": orig_cmd, "executor": executor})

            result = subprocess.run(  # noqa: S603 # nosec B603
                safe_cmd,
                capture_output=True,
                check=False,
                shell=False,
            )

            self._cache[key] = (result.returncode, result.stdout, result.stderr)
            self._cache.move_to_end(key)
            if len(self._cache) > self._MAX_CACHE:
                self._cache.popitem(last=False)

            self._tel.event(
                "procpool_executed",
                {
                    "cmd": orig_cmd,
                    "executor": executor,
                    "returncode": result.returncode,
                },
            )
            return result.returncode, result.stdout, result.stderr

        except BijuxError:
            self._tel.event(
                "procpool_execution_failed",
                {"cmd": cmd, "executor": executor, "error": "validation"},
            )
            raise
        except Exception as exc:
            failed_cmd = orig_cmd if "orig_cmd" in locals() else cmd
            self._tel.event(
                "procpool_execution_failed",
                {"cmd": failed_cmd, "executor": executor, "error": str(exc)},
            )
            raise BijuxError(f"Process-pool execution failed: {exc}") from exc

    def shutdown(self) -> None:
        """Gracefully shuts down the worker process pool."""
        self._exec.shutdown(wait=True)
        self._tel.event("procpool_shutdown", {})
        self._log.log("debug", "Process-pool shutdown")

    def get_status(self) -> dict[str, Any]:
        """Returns the current status of the process pool.

        Returns:
            dict[str, Any]: A dictionary containing status information, such as
                the number of commands processed (and cached).
        """
        return {"commands_processed": len(self._cache)}


def get_process_pool(
    logger: ObservabilityProtocol, telemetry: TelemetryProtocol
) -> ProcessPoolProtocol:
    """A factory function for creating a `ProcessPool` instance.

    This helper respects the `BIJUXCLI_MAX_WORKERS` environment variable to
    configure the number of worker processes.

    Args:
        logger (ObservabilityProtocol): The logging service instance.
        telemetry (TelemetryProtocol): The telemetry service instance.

    Returns:
        ProcessPoolProtocol: A configured instance of the process pool service.
    """
    workers = int(os.getenv("BIJUXCLI_MAX_WORKERS", "4"))
    return ProcessPool(logger, telemetry, max_workers=workers)


__all__ = ["get_process_pool", "ProcessPool"]
