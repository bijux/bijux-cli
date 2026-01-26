# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Tests for flag precedence and logging semantics."""

from __future__ import annotations

from typing import cast

from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import (
    FlagLayer,
    Flags,
    GlobalCLIConfig,
    resolve_effective_config,
    resolve_log_policy,
    resolve_output_flags,
    validate_cli_flags,
)


def test_resolve_output_flags_default() -> None:
    result = resolve_output_flags(
        quiet=False,
        pretty=False,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
    )
    assert result.log_level is LogLevel.INFO
    assert result.pretty is False
    assert result.color is ColorMode.AUTO


def test_resolve_output_flags_debug_overrides() -> None:
    result = resolve_output_flags(
        quiet=False,
        pretty=False,
        log_level=LogLevel.DEBUG,
        color=ColorMode.ALWAYS,
    )
    assert result.log_level is LogLevel.DEBUG
    assert result.pretty is False
    assert result.color is ColorMode.ALWAYS


def test_resolve_output_flags_quiet_wins() -> None:
    result = resolve_output_flags(
        quiet=True,
        pretty=True,
        log_level=LogLevel.DEBUG,
        color=ColorMode.AUTO,
    )
    assert result.log_level is LogLevel.ERROR
    assert result.pretty is True


def test_resolve_effective_config_prefers_cli_layer() -> None:
    effective = resolve_effective_config(
        cli=FlagLayer(color=ColorMode.ALWAYS, format=OutputFormat.JSON),
        env=FlagLayer(),
        file=FlagLayer(),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.YAML,
        ),
    )
    assert effective.flags.format is OutputFormat.JSON
    assert effective.flags.color is ColorMode.ALWAYS


def test_resolve_log_policy_thresholds() -> None:
    policy = resolve_log_policy(LogLevel.WARNING)
    assert policy.telemetry_verbosity == 1
    assert policy.show_traceback is False


def test_validate_cli_flags_reports_invalid_format() -> None:
    config = FlagLayer(format=OutputFormat("json"))
    errors = validate_cli_flags(
        GlobalCLIConfig(
            help=False,
            flags=config,
            args=("--format", "toml"),
            errors=(),
        )
    )
    assert errors == ()


def test_validate_cli_flags_rejects_unknown_format() -> None:
    config = FlagLayer(format=cast(OutputFormat, "toml"))
    errors = validate_cli_flags(
        GlobalCLIConfig(
            help=False,
            flags=config,
            args=("--format", "toml"),
            errors=(),
        )
    )
    assert any(err.failure == "invalid_format" for err in errors)
