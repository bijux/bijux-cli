# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Extra enum coverage tests for case-insensitive lookup."""

from __future__ import annotations

from enum import Enum

import pytest

from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat


@pytest.mark.parametrize(
    ("value", "expected"),
    [
        ("JSON", OutputFormat.JSON),
        ("yaml", OutputFormat.YAML),
        ("AUTO", ColorMode.AUTO),
        ("always", ColorMode.ALWAYS),
        ("INFO", LogLevel.INFO),
        ("error", LogLevel.ERROR),
    ],
)
def test_enum_missing_case_insensitive(value: str, expected: Enum) -> None:
    assert type(expected)(value) is expected


def test_enum_missing_raises_for_invalid_value() -> None:
    with pytest.raises(ValueError, match="OutputFormat"):
        OutputFormat("toml")
