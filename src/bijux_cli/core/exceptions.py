# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the custom exception hierarchy for the Bijux CLI.

This module provides a set of custom exception classes that inherit from the
base `BijuxError`. This hierarchy allows for more specific error handling
and helps standardize error reporting throughout the application. Each
exception can carry contextual information, such as the command that was
running when the error occurred.
"""

from __future__ import annotations


class BijuxError(Exception):
    """Base exception for all custom errors in the Bijux CLI.

    Attributes:
        command (str | None): The name of the command being executed when
            the error occurred.
        http_status (int): An HTTP-like status code used to derive the
            final CLI exit code.

    Args:
        message (str): The human-readable error message.
        command (str | None): The name of the command.
        http_status (int | None): The associated status code.
    """

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize the base BijuxError exception."""
        self.command = command
        self.http_status = http_status if http_status is not None else 500
        super().__init__(message)


class ServiceError(BijuxError):
    """Raised for service-related failures.

    This exception is used for errors originating from core services like
    Observability, Telemetry, or the Registry.

    Args:
        message (str): The human-readable error message.
        command (str | None): The name of the command.
        http_status (int | None): The associated status code. Defaults to 500.
    """

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize the ServiceError exception."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 500,
        )


class CommandError(BijuxError):
    """Raised for command execution failures.

    This exception is used for command-specific errors, such as invalid
    arguments or missing resources, that represent a client-side error.

    Args:
        message (str): The human-readable error message.
        command (str | None): The name of the command.
        http_status (int | None): The associated status code. Defaults to 400.
    """

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize the CommandError exception."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 400,
        )


class ConfigError(BijuxError):
    """Raised for configuration loading or parsing failures.

    Args:
        message (str): The human-readable error message.
        command (str | None): The name of the command.
        http_status (int | None): The associated status code. Defaults to 400.
    """

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize the ConfigError exception."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 400,
        )


class ValidationError(BijuxError):
    """Raised for validation failures (deprecated).

    Note:
        This exception is deprecated. Use `CommandError` for new code.

    Args:
        message (str): The human-readable error message.
        command (str | None): The name of the command.
        http_status (int | None): The associated status code. Defaults to 400.
    """

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize the ValidationError exception."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 400,
        )


class CliTimeoutError(BijuxError):
    """Raised for timeout errors (deprecated).

    Note:
        This exception is deprecated. Use `CommandError` or Python's built-in
        `TimeoutError` for new code.

    Args:
        message (str): The human-readable error message.
        command (str | None): The name of the command.
        http_status (int | None): The associated status code. Defaults to 504.
    """

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize the CliTimeoutError exception."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 504,
        )


__all__ = [
    "BijuxError",
    "ServiceError",
    "CommandError",
    "ConfigError",
    "ValidationError",
    "CliTimeoutError",
]
