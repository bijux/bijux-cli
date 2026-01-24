# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Flag/env/config precedence helpers."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass, field
from typing import Any

from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat


@dataclass(frozen=True)
class GlobalCLIConfig:
    """Immutable container for parsed global CLI flags."""

    help: bool
    flags: FlagLayer
    args: tuple[str, ...]
    errors: tuple[FlagError, ...]


@dataclass(frozen=True)
class FlagError:
    """Structured error for flag parsing/validation."""

    message: str
    failure: str
    flag: str


@dataclass(frozen=True)
class Flags:
    """Resolved flag bundle for logging/output behavior."""

    quiet: bool
    log_level: LogLevel
    color: ColorMode
    format: OutputFormat


@dataclass(frozen=True)
class FlagLayer:
    """Optional flag layer for precedence resolution."""

    log_level: LogLevel | None = None
    color: ColorMode | None = None
    format: OutputFormat | None = None
    quiet: bool | None = None


@dataclass(frozen=True)
class EffectiveConfig:
    """Resolved output/logging flags after precedence and normalization."""

    flags: Flags


@dataclass(frozen=True)
class ExecutionPolicy:
    """Resolved execution policy shared across CLI/service boundaries."""

    output_format: OutputFormat
    color: ColorMode
    quiet: bool
    verbose: bool
    verbose_level: int
    log_level: LogLevel
    log_policy: LogPolicy = field(init=False)
    pretty: bool = True
    include_runtime: bool = False

    def __post_init__(self) -> None:
        """Backfill log policy when constructed directly."""
        object.__setattr__(self, "log_policy", resolve_log_policy(self.log_level))


@dataclass(frozen=True)
class OutputConfig:
    """Resolved output/logging configuration for services."""

    include_runtime: bool
    pretty: bool
    log_level: LogLevel
    color: ColorMode
    log_policy: LogPolicy


@dataclass(frozen=True)
class LogPolicy:
    """Typed logging policy derived from a log level threshold."""

    level: LogLevel
    show_internal: bool
    show_traceback: bool
    pretty_default: bool
    telemetry_verbosity: int


_LOG_RANK: dict[LogLevel, int] = {
    LogLevel.TRACE: 5,
    LogLevel.DEBUG: 10,
    LogLevel.INFO: 20,
    LogLevel.WARNING: 30,
    LogLevel.ERROR: 40,
    LogLevel.CRITICAL: 50,
}


def _log_rank(level: LogLevel) -> int:
    """Return a comparable rank for log levels."""
    return _LOG_RANK.get(level, _LOG_RANK[LogLevel.INFO])


def resolve_log_policy(log_level: LogLevel) -> LogPolicy:
    """Derive logging policy from a level threshold."""
    rank = _log_rank(log_level)
    debug_rank = _log_rank(LogLevel.DEBUG)
    info_rank = _log_rank(LogLevel.INFO)
    warn_rank = _log_rank(LogLevel.WARNING)
    if rank <= debug_rank:
        telemetry = 3
    elif rank <= info_rank:
        telemetry = 2
    elif rank <= warn_rank:
        telemetry = 1
    else:
        telemetry = 0
    return LogPolicy(
        level=log_level,
        show_internal=rank <= debug_rank,
        show_traceback=rank <= debug_rank,
        pretty_default=rank <= info_rank,
        telemetry_verbosity=telemetry,
    )


def _coerce_verbose(value: Any) -> int:
    """Normalize verbosity inputs into a non-negative integer level."""
    if isinstance(value, bool):
        return 1 if value else 0
    if isinstance(value, int):
        return max(0, value)
    if isinstance(value, str) and value.strip().isdigit():
        return int(value.strip())
    return 0


def validate_cli_flags(
    config: GlobalCLIConfig, parse_errors: Sequence[FlagError] | None = None
) -> tuple[FlagError, ...]:
    """Validate raw CLI flags without applying behavior."""
    errors: list[FlagError] = list(parse_errors or config.errors)
    flags = config.flags
    if flags.format is not None and flags.format not in (
        OutputFormat.JSON,
        OutputFormat.YAML,
    ):
        errors.append(
            FlagError(
                message="Invalid output format.",
                failure="invalid_format",
                flag="--format",
            )
        )
    if flags.color is not None and flags.color not in (
        ColorMode.AUTO,
        ColorMode.ALWAYS,
        ColorMode.NEVER,
    ):
        errors.append(
            FlagError(
                message="Invalid color mode.",
                failure="invalid_color",
                flag="--color",
            )
        )
    if flags.log_level is not None and flags.log_level not in (
        LogLevel.TRACE,
        LogLevel.DEBUG,
        LogLevel.INFO,
    ):
        errors.append(
            FlagError(
                message="Invalid log level.",
                failure="invalid_log_level",
                flag="--log-level",
            )
        )
    return tuple(errors)


def _pick_value(
    cli: FlagLayer,
    env: FlagLayer,
    file: FlagLayer,
    defaults: Flags,
) -> Flags:
    """Resolve precedence across four layers with first-set wins."""

    def pick(attr: str, fallback: Any) -> Any:
        for source in (cli, env, file):
            value = getattr(source, attr)
            if value is not None:
                return value
        return fallback

    return Flags(
        quiet=bool(pick("quiet", defaults.quiet)),
        log_level=pick("log_level", defaults.log_level),
        color=pick("color", defaults.color),
        format=pick("format", defaults.format),
    )


def resolve_effective_config(
    cli: FlagLayer,
    env: FlagLayer,
    file: FlagLayer,
    defaults: Flags,
) -> EffectiveConfig:
    """Resolve flag/env/config precedence into a single effective config.

    Algebraic laws:
      - Left-identity: resolve(cli, env, file, defaults) equals resolve(cli, empty, empty, defaults)
      - Right-identity: resolve(empty, empty, empty, defaults) equals defaults
      - Idempotence: resolve(a, a, a, defaults) equals resolve(a, empty, empty, defaults)
    """
    flags = _pick_value(cli, env, file, defaults)
    if flags.quiet:
        flags = Flags(
            quiet=True,
            log_level=LogLevel.ERROR,
            color=flags.color,
            format=flags.format,
        )
    return EffectiveConfig(flags=flags)


def resolve_execution_policy(effective: EffectiveConfig) -> ExecutionPolicy:
    """Create an immutable execution policy from resolved config."""
    flags = effective.flags
    log_policy = resolve_log_policy(flags.log_level)
    return ExecutionPolicy(
        output_format=flags.format,
        color=flags.color,
        quiet=flags.quiet,
        verbose=False,
        verbose_level=0,
        log_level=flags.log_level,
        pretty=log_policy.pretty_default,
        include_runtime=log_policy.show_internal,
    )


def default_execution_policy() -> ExecutionPolicy:
    """Return the default execution policy without DI."""
    defaults = Flags(
        quiet=False,
        log_level=LogLevel.INFO,
        color=ColorMode.AUTO,
        format=OutputFormat.JSON,
    )
    return resolve_execution_policy(EffectiveConfig(flags=defaults))


def resolve_output_flags(
    *,
    quiet: bool,
    pretty: bool,
    log_level: LogLevel = LogLevel.INFO,
    color: ColorMode = ColorMode.AUTO,
) -> OutputConfig:
    """Resolve logging/color/pretty flags from a single source of truth."""
    effective = resolve_effective_config(
        cli=FlagLayer(
            quiet=quiet,
            log_level=log_level,
            color=color,
            format=OutputFormat.JSON,
        ),
        env=FlagLayer(),
        file=FlagLayer(),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
    )
    log_policy = resolve_log_policy(effective.flags.log_level)
    return OutputConfig(
        include_runtime=False,
        pretty=pretty,
        log_level=effective.flags.log_level,
        color=effective.flags.color,
        log_policy=log_policy,
    )
