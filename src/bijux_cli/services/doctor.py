# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the concrete implementation of the CLI health check service.

This module defines the `Doctor` class, which implements the `DoctorProtocol`.
It is responsible for performing diagnostic health checks on the application
and its environment.
"""

from __future__ import annotations

from bijux_cli.contracts import DoctorProtocol


class Doctor(DoctorProtocol):
    """An implementation of the health check service.

    This class provides a simple health check method that can be extended in the
    future to verify dependencies, such as database connections or external API
    reachability.
    """

    def check_health(self) -> str:
        """Performs a basic health check on the application.

        Returns:
            str: A string indicating the health status. Currently always
                returns "healthy".
        """
        return "healthy"


__all__ = ["Doctor"]
