# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Regression matrix for global CLI flag parsing and precedence."""

from __future__ import annotations

import pytest

from bijux_cli.cli.core.flags import collect_global_flag_errors, parse_global_flags
from bijux_cli.cli.core.command import normalize_format
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import FlagLayer, Flags, resolve_effective_config


@pytest.mark.parametrize(
    ("argv", "expected"),
    [
        (["--log-level"], "missing_argument"),
        (["--color"], "missing_argument"),
        (["--format"], "missing_argument"),
    ],
)
def test_collect_missing_flag_values(argv: list[str], expected: str) -> None:
    errors = collect_global_flag_errors(argv)
    assert errors and errors[0].failure == expected


@pytest.mark.parametrize(
    ("argv", "failure"),
    [
        (["--log-level", "nope"], "invalid_log_level"),
        (["--color", "nope"], "invalid_color"),
        (["--format", "nope"], "invalid_format"),
    ],
)
def test_collect_invalid_flag_values(argv: list[str], failure: str) -> None:
    errors = collect_global_flag_errors(argv)
    assert errors and errors[0].failure == failure


def test_parse_global_flags_extracts_values() -> None:
    flags = parse_global_flags(["--quiet", "--log-level", "debug", "--color", "never"])
    assert flags.quiet is True
    assert flags.log_level is LogLevel.DEBUG
    assert flags.color is ColorMode.NEVER


def test_env_overrides_config_in_precedence() -> None:
    effective = resolve_effective_config(
        cli=FlagLayer(),
        env=FlagLayer(log_level=LogLevel.DEBUG),
        file=FlagLayer(log_level=LogLevel.INFO),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
    )
    assert effective.flags.log_level is LogLevel.DEBUG


def test_cli_overrides_env_and_config() -> None:
    effective = resolve_effective_config(
        cli=FlagLayer(format=OutputFormat.YAML),
        env=FlagLayer(format=OutputFormat.JSON),
        file=FlagLayer(format=OutputFormat.JSON),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
    )
    assert effective.flags.format is OutputFormat.YAML


def test_quiet_normalizes_log_level() -> None:
    effective = resolve_effective_config(
        cli=FlagLayer(quiet=True, log_level=LogLevel.INFO),
        env=FlagLayer(),
        file=FlagLayer(),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
    )
    assert effective.flags.quiet is True
    assert effective.flags.log_level is LogLevel.ERROR


@pytest.mark.parametrize(
    ("raw", "expected"),
    [
        (" json ", OutputFormat.JSON),
        ("YAML", OutputFormat.YAML),
        ("", None),
        ("toml", None),
    ],
)
def test_format_normalization_edge_cases(raw: str, expected: OutputFormat | None) -> None:
    assert normalize_format(raw) == expected
