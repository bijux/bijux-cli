# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for a worker process pool service.

This module specifies the `ProcessPoolProtocol`, a formal interface that any
service managing a pool of worker processes for command execution must
implement. This allows for abstracting the details of subprocess management.
"""

from __future__ import annotations

from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class ProcessPoolProtocol(Protocol):
    """Defines the contract for a worker process pool.

    This interface specifies the methods for running commands in isolated
    worker processes and managing the lifecycle of the pool.
    """

    def run(self, cmd: list[str], *, executor: str) -> tuple[int, bytes, bytes]:
        """Executes a command in a worker process.

        Args:
            cmd (list[str]): A list of command arguments to execute.
            executor (str): The name or identifier of the executor to use.

        Returns:
            tuple[int, bytes, bytes]: A tuple containing the return code,
                standard output (stdout), and standard error (stderr) as bytes.
        """
        ...

    def shutdown(self) -> None:
        """Shuts down the process pool and releases all associated resources."""
        ...

    def get_status(self) -> dict[str, Any]:
        """Returns the current status of the process pool.

        Returns:
            dict[str, Any]: A dictionary containing status information, such as
                the number of active workers, queue size, etc.
        """
        ...
