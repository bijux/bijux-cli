# SPDX-License-Identifier: Apache-2.0
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


class UserInputError(BijuxError):
    """Raised for user input and validation failures."""

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize a user input error."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 400,
        )


class ConfigError(UserInputError):
    """Raised for configuration loading or parsing failures."""


class PluginError(BijuxError):
    """Raised for plugin lifecycle and compatibility failures."""

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize a plugin error."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 400,
        )


class InternalError(BijuxError):
    """Raised for internal failures."""

    def __init__(
        self,
        message: str,
        *,
        command: str | None = None,
        http_status: int | None = None,
    ) -> None:
        """Initialize an internal error."""
        super().__init__(
            message,
            command=command,
            http_status=http_status if http_status is not None else 500,
        )


__all__ = [
    "BijuxError",
    "UserInputError",
    "PluginError",
    "InternalError",
    "ConfigError",
]
