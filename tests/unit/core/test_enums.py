# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Unit tests for the core enums module."""

from __future__ import annotations

from typing import Any

import pytest

from bijux_cli.core.enums import OutputFormat


def test_members() -> None:
    """Test that the enum members have the correct string values."""
    assert OutputFormat.JSON.value == "json"
    assert OutputFormat.YAML.value == "yaml"


@pytest.mark.parametrize(
    ("value", "expected"),
    [
        ("json", OutputFormat.JSON),
        ("JSON", OutputFormat.JSON),
        ("Json", OutputFormat.JSON),
        ("yaml", OutputFormat.YAML),
        ("YAML", OutputFormat.YAML),
        ("Yaml", OutputFormat.YAML),
    ],
)
def test_case_insensitive_lookup(value: str, expected: OutputFormat) -> None:
    """Test that enum lookup is case-insensitive for string values."""
    assert OutputFormat(value) is expected


@pytest.mark.parametrize("invalid_value", ["invalid", "XML", 123, None])
def test_invalid_lookup(invalid_value: Any) -> None:
    """Test that looking up an invalid value raises a ValueError."""
    with pytest.raises(ValueError, match=r"is not a valid OutputFormat"):
        OutputFormat(invalid_value)


def test_member_passthrough() -> None:
    """Test that passing an enum member to the constructor returns the same member."""
    assert OutputFormat(OutputFormat.JSON) is OutputFormat.JSON


def test_all_exported() -> None:
    """Test that the module's __all__ contains only the OutputFormat enum."""
    from bijux_cli.core.enums import __all__

    assert __all__ == ["OutputFormat"]


def parse_output_format(value: Any) -> OutputFormat:
    """Demonstrate a helper function for parsing the OutputFormat enum."""
    if not isinstance(value, str):
        raise ValueError("format must be a string")
    return OutputFormat(value)
