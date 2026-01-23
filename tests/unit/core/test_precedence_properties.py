# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Property tests for precedence resolution."""

from __future__ import annotations

from hypothesis import given
from hypothesis import strategies as st

from bijux_cli.cli.flags import parse_global_flags
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import (
    FlagLayer,
    Flags,
    resolve_effective_config,
    validate_cli_flags,
)

_log_levels = st.sampled_from(list(LogLevel))
_colors = st.sampled_from(list(ColorMode))
_formats = st.sampled_from(list(OutputFormat))


@given(
    cli_quiet=st.one_of(st.none(), st.just(True)),
    env_quiet=st.one_of(st.none(), st.booleans()),
    file_quiet=st.one_of(st.none(), st.booleans()),
)
def test_left_identity_for_quiet(
    cli_quiet: bool | None, env_quiet: bool | None, file_quiet: bool | None
) -> None:
    defaults = Flags(
        quiet=False,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
        format=OutputFormat.JSON,
    )
    argv: list[str] = []
    if cli_quiet:
        argv.append("--quiet")
    parsed = parse_global_flags(argv)
    assert validate_cli_flags(parsed) == ()
    cli = parsed.flags
    env = FlagLayer(quiet=env_quiet)
    file = FlagLayer(quiet=file_quiet)

    eff_full = resolve_effective_config(cli=cli, env=env, file=file, defaults=defaults)
    expected = (
        cli_quiet
        if cli_quiet is not None
        else env_quiet
        if env_quiet is not None
        else file_quiet
        if file_quiet is not None
        else defaults.quiet
    )
    assert eff_full.flags.quiet == expected


@given(
    color=_colors,
    log_level=_log_levels,
    fmt=_formats,
)
def test_right_identity_defaults_only(
    color: ColorMode, log_level: LogLevel, fmt: OutputFormat
) -> None:
    defaults = Flags(
        quiet=False,
        log_level=log_level,
        color=color,
        format=fmt,
    )
    parsed = parse_global_flags([])
    assert validate_cli_flags(parsed) == ()
    effective = resolve_effective_config(
        cli=parsed.flags, env=FlagLayer(), file=FlagLayer(), defaults=defaults
    )
    assert effective.flags == defaults


@given(
    color=_colors,
    log_level=_log_levels,
    fmt=_formats,
)
def test_idempotence_when_layers_equal(
    color: ColorMode, log_level: LogLevel, fmt: OutputFormat
) -> None:
    defaults = Flags(
        quiet=False,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
        format=OutputFormat.JSON,
    )
    argv = [
        "--quiet",
        "--log-level",
        log_level.value,
        "--color",
        color.value,
        "--format",
        fmt.value,
    ]
    parsed = parse_global_flags(argv)
    assert validate_cli_flags(parsed) == ()
    layer = FlagLayer(
        quiet=True,
        log_level=log_level,
        color=color,
        format=fmt,
    )
    eff_a = resolve_effective_config(
        cli=parsed.flags, env=layer, file=layer, defaults=defaults
    )
    eff_b = resolve_effective_config(
        cli=parsed.flags, env=FlagLayer(), file=FlagLayer(), defaults=defaults
    )
    assert eff_a == eff_b


@given(
    log_level=_log_levels,
    fmt=_formats,
)
def test_quiet_dominates_log_level(log_level: LogLevel, fmt: OutputFormat) -> None:
    """Quiet should force ERROR regardless of requested log level."""
    defaults = Flags(
        quiet=False,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
        format=OutputFormat.JSON,
    )
    argv = ["--quiet", "--log-level", log_level.value, "--format", fmt.value]
    parsed = parse_global_flags(argv)
    assert validate_cli_flags(parsed) == ()
    effective = resolve_effective_config(
        cli=parsed.flags, env=FlagLayer(), file=FlagLayer(), defaults=defaults
    )
    assert effective.flags.quiet is True
    assert effective.flags.log_level is LogLevel.ERROR
