# SPDX-License-Identifier: MIT
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


__all__ = ["OutputFormat"]
