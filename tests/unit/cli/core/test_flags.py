# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Tests for global CLI flag parsing and errors."""

from __future__ import annotations

from bijux_cli.cli.core.flags import collect_global_flag_errors, parse_global_flags
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat


def test_parse_global_flags_invalid_values_are_ignored() -> None:
    flags = parse_global_flags(
        ["--format", "bogus", "--color", "bad", "--log-level", "nope"]
    )
    assert flags.format is None
    assert flags.color is None
    assert flags.log_level is None


def test_parse_global_flags_missing_values() -> None:
    flags = parse_global_flags(["--log-level", "--color", "--format"])
    assert flags.log_level is None
    assert flags.color is None
    assert flags.format is None


def test_collect_global_flag_errors_for_log_level() -> None:
    errors = collect_global_flag_errors(["--log-level"])
    assert errors
    assert errors[0].failure == "missing_argument"
    errors = collect_global_flag_errors(["--log-level", "nope"])
    assert errors
    assert errors[0].failure == "invalid_log_level"


def test_collect_global_flag_errors_for_color() -> None:
    errors = collect_global_flag_errors(["--color"])
    assert errors
    assert errors[0].failure == "missing_argument"
    errors = collect_global_flag_errors(["--color", "neon"])
    assert errors
    assert errors[0].failure == "invalid_color"


def test_collect_global_flag_errors_for_format() -> None:
    errors = collect_global_flag_errors(["--format"])
    assert errors
    assert errors[0].failure == "missing_argument"
    errors = collect_global_flag_errors(["--format", "toml"])
    assert errors
    assert errors[0].failure == "invalid_format"


def test_collect_global_flag_errors_valid_values() -> None:
    errors = collect_global_flag_errors(
        ["--log-level", "info", "--color", "auto", "--format", "json"]
    )
    assert errors == ()


def test_parse_global_flags_success() -> None:
    flags = parse_global_flags(
        ["--quiet", "--log-level", "debug", "--color", "never", "--format", "json"]
    )
    assert flags.quiet is True
    assert flags.log_level == LogLevel.DEBUG
    assert flags.color == ColorMode.NEVER
    assert flags.format == OutputFormat.JSON
