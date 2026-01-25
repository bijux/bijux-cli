# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Process execution adapters."""

from __future__ import annotations

from collections import OrderedDict
from concurrent.futures import ProcessPoolExecutor
import os
import shutil
import subprocess  # nosec B404
from typing import Any


def validate_command(cmd: list[str], *, allowed_commands: list[str]) -> list[str]:
    """Validates a command and its arguments against a whitelist."""
    if not cmd:
        raise ValueError("invalid command: empty")

    cmd_name = os.path.basename(cmd[0])
    if cmd_name not in allowed_commands:
        raise ValueError(
            f"invalid command {cmd_name!r} not in allowed list: {allowed_commands}"
        )
    resolved_cmd_path = shutil.which(cmd[0])
    if not resolved_cmd_path:
        raise ValueError(f"Command not found or not executable: {cmd[0]!r}")
    if os.path.basename(resolved_cmd_path) != cmd_name:
        raise ValueError(f"Disallowed command path: {cmd[0]!r}")
    cmd[0] = resolved_cmd_path
    forbidden = set(";|&><`!")
    for arg in cmd[1:]:
        if any(ch in arg for ch in forbidden):
            raise ValueError(f"Unsafe argument: {arg!r}")
    return cmd


class ProcessPool:
    """Executes validated commands in a worker pool with an LRU cache."""

    _MAX_CACHE = 1000

    def __init__(
        self,
        observability: Any,
        telemetry: Any,
        max_workers: int,
        allowed_commands: list[str],
    ) -> None:
        """Initialize the process pool executor."""
        self._exec = ProcessPoolExecutor(max_workers=max_workers)
        self._log = observability
        self._tel = telemetry
        self._allowed_commands = allowed_commands
        self._cache: OrderedDict[tuple[str, ...], tuple[int, bytes, bytes]] = (
            OrderedDict()
        )

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Run a validated command via the process pool."""
        key = tuple(cmd)
        if key in self._cache:
            self._log.log("debug", "Process-pool cache hit", extra={"cmd": cmd})
            self._tel.event("procpool_cache_hit", {"cmd": cmd, "executor": executor})
            self._cache.move_to_end(key)
            return self._cache[key]

        orig_cmd = list(cmd)
        try:
            validate = __import__(
                "bijux_cli.infra.process", fromlist=["validate_command"]
            ).validate_command
            safe_cmd = validate(cmd, allowed_commands=self._allowed_commands)
        except ValueError:
            self._tel.event(
                "procpool_execution_failed",
                {"cmd": cmd, "executor": executor, "error": "validation"},
            )
            raise

        try:
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
        except Exception as exc:
            self._tel.event(
                "procpool_execution_failed",
                {"cmd": orig_cmd, "executor": executor, "error": str(exc)},
            )
            raise RuntimeError(f"Process-pool execution failed: {exc}") from exc

    def shutdown(self) -> None:
        """Shutdown the process pool and emit telemetry."""
        self._exec.shutdown(wait=True)
        self._tel.event("procpool_shutdown", {})
        self._log.log("debug", "Process-pool shutdown")

    def get_status(self) -> dict[str, Any]:
        """Return basic status for the process pool."""
        return {"commands_processed": len(self._cache)}


__all__ = ["ProcessPool", "validate_command"]
