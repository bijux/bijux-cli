# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""CLI output helpers for resolving config and emitting payloads."""

from __future__ import annotations

from collections.abc import Callable

from bijux_cli.cli.emit import emit_and_exit, emit_error_and_exit
from bijux_cli.cli.validation import validate_common_flags
from bijux_cli.core.enums import ColorMode, LogLevel, OutputFormat
from bijux_cli.core.precedence import (
    ExecutionPolicy,
    FlagLayer,
    Flags,
    resolve_effective_config,
    resolve_execution_policy,
)


def _resolve_policy() -> ExecutionPolicy:
    """Resolve the shared execution policy from DI or defaults."""
    try:
        from bijux_cli.core.di import DIContainer

        policy = DIContainer.current().resolve(ExecutionPolicy)
        if not isinstance(policy, ExecutionPolicy):
            raise TypeError("ExecutionPolicy not available")
        return policy
    except Exception:
        effective = resolve_effective_config(
            cli=FlagLayer(),
            env=FlagLayer(),
            file=FlagLayer(),
            defaults=Flags(
                quiet=False,
                log_level=LogLevel.INFO,
                color=ColorMode.AUTO,
                format=OutputFormat.JSON,
            ),
        )
        return resolve_execution_policy(effective)


def get_execution_policy() -> ExecutionPolicy:
    """Return the shared execution policy for CLI commands."""
    return _resolve_policy()


def resolve_command_config(
    *,
    command: str,
    quiet: bool,
    verbose: bool,
    log_level: str,
    fmt: str,
    pretty: bool,
) -> tuple[ExecutionPolicy, OutputFormat, OutputFormat]:
    """Resolve the shared policy for a command invocation."""
    _ = (quiet, verbose, log_level, fmt, pretty)
    effective = get_execution_policy()
    format_source = fmt if isinstance(fmt, str) else effective.output_format.value
    output_format = validate_common_flags(
        format_source,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
    )
    return effective, output_format, output_format


def new_run_command(
    command_name: str,
    payload_builder: Callable[[bool], object],
    quiet: bool,
    verbose: bool,
    fmt: OutputFormat,
    pretty: bool,
    log_level: str,
    exit_code: int = 0,
) -> None:
    """Build and emit a command payload using resolved config."""
    from bijux_cli.core.di import DIContainer
    from bijux_cli.infra.contracts import Emitter
    from bijux_cli.services.contracts import TelemetryProtocol

    _ = (quiet, verbose, fmt, pretty, log_level)
    DIContainer.current().resolve(Emitter)
    DIContainer.current().resolve(TelemetryProtocol)

    resolved = get_execution_policy()
    include_runtime = resolved.include_runtime

    format_source = fmt
    output_format = validate_common_flags(
        format_source,
        command_name,
        resolved.quiet,
        include_runtime=include_runtime,
    )
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
            debug=(resolved.log_level == LogLevel.DEBUG),
        )
    else:
        emit_and_exit(
            payload=payload,
            fmt=output_format,
            effective_pretty=effective_pretty,
            verbose=resolved.verbose,
            debug=(resolved.log_level == LogLevel.DEBUG),
            quiet=resolved.quiet,
            command=command_name,
            exit_code=exit_code,
        )


__all__ = ["get_execution_policy", "resolve_command_config", "new_run_command"]
