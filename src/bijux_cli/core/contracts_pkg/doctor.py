# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the contract for the CLI health check and diagnostics service.

This module specifies the `DoctorProtocol`, a formal interface that any
service responsible for running diagnostic health checks on the CLI and its
environment must implement.
"""

from __future__ import annotations

from typing import Protocol, TypeVar, runtime_checkable

T = TypeVar("T")


@runtime_checkable
class DoctorProtocol(Protocol):
    """Defines the contract for CLI health checks.

    This interface specifies the methods for performing system health checks
    and gathering diagnostic information about the CLI and its environment.
    """

    def check_health(self) -> str:
        """Performs health checks and returns a status string.

        Returns:
            str: A string describing the overall health status of the CLI.
        """
        ...
