# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""CLI output helpers for resolving config and emitting payloads."""

from __future__ import annotations

from collections.abc import Callable, Mapping
from typing import Any

from bijux_cli.cli.emit import emit_and_exit, emit_error_and_exit
from bijux_cli.cli.validation import validate_common_flags
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.precedence import EffectiveConfig


def effective_defaults() -> dict[str, Any]:
    """Return effective defaults for CLI output options."""
    try:
        from bijux_cli.app.di import DIContainer

        effective = DIContainer.current().resolve(EffectiveConfig)
        if not isinstance(effective, EffectiveConfig):
            raise TypeError("EffectiveConfig not available")
    except Exception:
        return {
            "quiet": False,
            "verbose": False,
            "pretty": True,
            "log_level": "info",
            "color": "auto",
            "format": "json",
            "json": False,
        }
    return {
        "quiet": effective.quiet,
        "verbose": effective.verbose_level > 0,
        "pretty": effective.pretty,
        "log_level": effective.log_level,
        "color": effective.color,
        "format": effective.fmt,
        "json": effective.json,
    }


def resolve_command_config(
    *,
    command: str,
    quiet: bool,
    verbose: bool,
    log_level: str,
    fmt: str,
    pretty: bool,
) -> tuple[EffectiveConfig, OutputFormat, str]:
    """Resolve CLI flags into an effective config and output format."""

    def _unwrap(value: object) -> object:
        return getattr(value, "default", value)

    quiet = bool(_unwrap(quiet))
    verbose = bool(_unwrap(verbose))
    log_level = str(_unwrap(log_level))
    fmt = str(_unwrap(fmt))
    pretty = bool(_unwrap(pretty))
    from bijux_cli.core.precedence import resolve_effective_config

    effective = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "pretty": pretty,
            "format": fmt,
            "log_level": log_level,
        },
        env={},
        file={},
        defaults={
            "quiet": False,
            "verbose": False,
            "pretty": True,
            "log_level": "info",
            "color": "auto",
            "format": "json",
            "json": False,
        },
    )
    format_lower = validate_common_flags(
        effective.fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
    )
    output_format = OutputFormat.YAML if format_lower == "yaml" else OutputFormat.JSON
    return effective, output_format, format_lower


def new_run_command(
    command_name: str,
    payload_builder: Callable[[bool], Mapping[str, object]],
    quiet: bool,
    verbose: bool,
    fmt: str,
    pretty: bool,
    log_level: str,
    exit_code: int = 0,
) -> None:
    """Build and emit a command payload using resolved config."""
    from bijux_cli.app.di import DIContainer
    from bijux_cli.core.contracts import Emitter
    from bijux_cli.core.precedence import resolve_effective_config
    from bijux_cli.services.contracts import TelemetryProtocol

    DIContainer.current().resolve(Emitter)
    DIContainer.current().resolve(TelemetryProtocol)

    resolved = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "pretty": pretty,
            "format": fmt,
            "log_level": log_level,
        },
        env={},
        file={},
        defaults=effective_defaults(),
    )
    include_runtime = resolved.include_runtime

    format_lower = validate_common_flags(
        resolved.fmt,
        command_name,
        resolved.quiet,
        include_runtime=include_runtime,
    )
    output_format = OutputFormat.YAML if format_lower == "yaml" else OutputFormat.JSON
    effective_pretty = resolved.pretty
    try:
        payload = payload_builder(include_runtime)
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command_name,
            fmt=output_format,
            quiet=resolved.quiet,
            include_runtime=include_runtime,
            debug=(resolved.log_level == "debug"),
        )
    else:
        emit_and_exit(
            payload=payload,
            fmt=output_format,
            effective_pretty=effective_pretty,
            verbose=resolved.verbose_level > 0,
            debug=(resolved.log_level == "debug"),
            quiet=resolved.quiet,
            command=command_name,
            exit_code=exit_code,
        )


__all__ = ["effective_defaults", "resolve_command_config", "new_run_command"]
