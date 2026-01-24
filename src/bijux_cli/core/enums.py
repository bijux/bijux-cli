# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Defines the output format enumeration for the Bijux CLI.

This module provides the `OutputFormat` enum, which represents the
supported structured output formats (JSON and YAML). Using an enum ensures
type safety and provides a single source of truth for format names. It also
includes a mechanism for case-insensitive matching.
"""

from __future__ import annotations

from enum import Enum


class OutputFormat(str, Enum):
    """Specifies the supported structured output formats for CLI responses.

    This enum supports case-insensitive matching, so `OutputFormat("JSON")` and
    `OutputFormat("yaml")` are both valid.
    """

    JSON = "json"
    YAML = "yaml"

    @classmethod
    def _missing_(cls, value: object) -> OutputFormat:
        """Handles case-insensitive lookup of enum members.

        This special method is called by the `Enum` metaclass when a value is
        not found. This implementation retries the lookup in lowercase.

        Args:
            value: The value being looked up.

        Returns:
            OutputFormat: The matching enum member.

        Raises:
            ValueError: If no matching member is found after converting the
                input value to lowercase.
        """
        if isinstance(value, str):
            value_lower = value.lower()
            for member in cls:
                if member.value == value_lower:
                    return member
        raise ValueError(f"{value} is not a valid {cls.__name__}")


class ColorMode(str, Enum):
    """Specifies terminal color handling for CLI output."""

    AUTO = "auto"
    ALWAYS = "always"
    NEVER = "never"

    @classmethod
    def _missing_(cls, value: object) -> ColorMode:
        """Handle case-insensitive lookup of color modes."""
        if isinstance(value, str):
            value_lower = value.lower()
            for member in cls:
                if member.value == value_lower:
                    return member
        raise ValueError(f"{value} is not a valid {cls.__name__}")


class LogLevel(str, Enum):
    """Logging level names for structured logging."""

    TRACE = "trace"
    DEBUG = "debug"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    CRITICAL = "critical"

    @classmethod
    def _missing_(cls, value: object) -> LogLevel:
        """Handle case-insensitive lookup of log levels."""
        if isinstance(value, str):
            value_lower = value.lower()
            for member in cls:
                if member.value == value_lower:
                    return member
        raise ValueError(f"{value} is not a valid {cls.__name__}")


class ExecutionMode(str, Enum):
    """Execution mode for the runtime kernel."""

    CLI = "cli"
    API = "api"
    REPL = "repl"


class ExitCode(int, Enum):
    """Standardized exit codes for command execution."""

    SUCCESS = 0
    ERROR = 1
    USAGE = 2
    ASCII = 3
    ABORTED = 130


class ErrorType(str, Enum):
    """High-level error categories for exit behavior."""

    USAGE = "usage"
    ASCII = "ascii"
    USER_INPUT = "user_input"
    CONFIG = "config"
    PLUGIN = "plugin"
    INTERNAL = "internal"
    ABORTED = "aborted"


__all__ = [
    "ColorMode",
    "OutputFormat",
    "LogLevel",
    "ExecutionMode",
    "ExitCode",
    "ErrorType",
]
