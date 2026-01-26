# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Runtime execution for the `help` command (IO + exit behavior)."""

from __future__ import annotations

from collections.abc import Callable
import io
import sys
import time

import typer

from bijux_cli.cli.color import resolve_click_color
from bijux_cli.cli.commands.help import (
    _HUMAN,
    _VALID_FORMATS,
    _build_help_intent,
    _build_help_payload,
    _find_target_command,
    _get_formatted_help,
)
from bijux_cli.cli.core.command import (
    contains_non_ascii_env,
    raise_exit_intent,
    record_history,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
)
from bijux_cli.cli.core.help_text import (
    HELP_FORMAT_HELP,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import (
    ErrorType,
    ExitCode,
    OutputFormat,
)
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.precedence import (
    EffectiveConfig,
    Flags,
    OutputConfig,
    default_execution_policy,
)
from bijux_cli.core.runtime import AsyncTyper

typer.core.rich = None  # type: ignore[attr-defined]


def _resolve_help_config() -> tuple[EffectiveConfig, OutputConfig]:
    """Resolve effective and output config for help handling."""
    try:
        effective = DIContainer.current().resolve(EffectiveConfig)
    except Exception:
        policy = default_execution_policy()
        effective = EffectiveConfig(
            flags=Flags(
                quiet=policy.quiet,
                log_level=policy.log_level,
                color=policy.color,
                format=policy.output_format,
            )
        )
    try:
        output = DIContainer.current().resolve(OutputConfig)
    except Exception:
        policy = default_execution_policy()
        output = OutputConfig(
            include_runtime=policy.log_policy.show_internal,
            pretty=policy.log_policy.pretty_default,
            log_level=effective.flags.log_level,
            color=effective.flags.color,
            format=effective.flags.format,
            log_policy=policy.log_policy,
        )
    return effective, output


def _emit_structured_help(
    *,
    command: str,
    payload: dict[str, object],
    output_format: OutputFormat,
    pretty: bool,
    emit_output: bool,
) -> None:
    """Emit structured help payload with history recording."""
    record_history(command, ExitCode.SUCCESS)
    if not emit_output:
        raise ExitIntentError(
            ExitIntent(
                code=ExitCode.SUCCESS,
                stream=None,
                payload=None,
                fmt=output_format,
                pretty=pretty,
                show_traceback=False,
            )
        )
    raise ExitIntentError(
        ExitIntent(
            code=ExitCode.SUCCESS,
            stream="stdout",
            payload=payload,
            fmt=output_format,
            pretty=pretty,
            show_traceback=False,
        )
    )


def _emit_human_help(
    *,
    emit_output: bool,
    color: bool,
    help_text_provider: Callable[[], str],
) -> None:
    """Emit human help output without building text in quiet mode."""
    if not emit_output:
        raise ExitIntentError(
            ExitIntent(
                code=ExitCode.SUCCESS,
                stream=None,
                payload=None,
                fmt=OutputFormat.JSON,
                pretty=False,
                show_traceback=False,
            )
        )
    text = help_text_provider()
    typer.echo(text, color=color, err=False)
    raise ExitIntentError(
        ExitIntent(
            code=ExitCode.SUCCESS,
            stream=None,
            payload=None,
            fmt=OutputFormat.JSON,
            pretty=False,
            show_traceback=False,
        )
    )


def _capture_help_text(help_text_provider: Callable[[], str]) -> str:
    """Capture help text without leaking human output to stdout."""
    buf = io.StringIO()
    original = sys.stdout
    sys.stdout = buf
    try:
        text = help_text_provider()
    finally:
        sys.stdout = original
    captured = buf.getvalue()
    if text.strip():
        return text
    return captured


def _override_fmt_from_argv(fmt: str) -> str:
    """Prefer an explicit format flag value when provided on the CLI."""
    if fmt.strip().lower() != _HUMAN:
        return fmt
    argv = sys.argv[1:]
    if "help" in argv:
        argv = argv[argv.index("help") + 1 :]
    for idx, arg in enumerate(argv):
        if arg in OPT_FORMAT and idx + 1 < len(argv):
            return argv[idx + 1]
    return fmt


help_app = AsyncTyper(
    name="help",
    add_completion=False,
    help="Show help for any CLI command or subcommand.",
    context_settings={
        "help_option_names": ["-h", "--help"],
        "ignore_unknown_options": True,
        "allow_extra_args": True,
        "allow_interspersed_args": True,
    },
)

ARGS = typer.Argument(None, help="Command path, e.g. 'config get'.")


@help_app.callback(invoke_without_command=True)
def help_callback(
    ctx: typer.Context,
    command_path: list[str] | None = ARGS,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option(_HUMAN, *OPT_FORMAT, help=HELP_FORMAT_HELP),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Entry point for the `bijux help` command."""
    _ = (quiet, pretty, log_level)
    started_at = time.perf_counter()
    effective, output = _resolve_help_config()
    emit_output = not effective.flags.quiet
    fmt = _override_fmt_from_argv(fmt)

    if "-h" in sys.argv or "--help" in sys.argv:
        all_args = sys.argv[2:]
        known_flags_with_args = set(OPT_FORMAT)
        path_tokens = []
        i = 0
        while i < len(all_args):
            arg = all_args[i]
            if arg in known_flags_with_args:
                i += 2
            elif arg.startswith("-"):
                i += 1
            else:
                path_tokens.append(arg)
                i += 1

        target = _find_target_command(ctx, path_tokens) or _find_target_command(ctx, [])
        if target:
            target_cmd, target_ctx = target
            help_text = _get_formatted_help(target_cmd, target_ctx)
            if emit_output:
                typer.echo(
                    help_text,
                    color=resolve_click_color(quiet=False, fmt=None),
                    err=False,
                )
            raise ExitIntentError(
                ExitIntent(
                    code=ExitCode.SUCCESS,
                    stream=None,
                    payload=None,
                    fmt=OutputFormat.JSON,
                    pretty=False,
                    show_traceback=False,
                )
            )
        raise ExitIntentError(
            ExitIntent(
                code=ExitCode.SUCCESS,
                stream=None,
                payload=None,
                fmt=OutputFormat.JSON,
                pretty=False,
                show_traceback=False,
            )
        )

    tokens = command_path or []
    command = "help"
    intent = _build_help_intent(tokens, fmt, effective, output)

    if intent.fmt_lower != "human":
        validate_common_flags(
            intent.format_value or OutputFormat.JSON,
            command,
            intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )

    if intent.fmt_lower not in _VALID_FORMATS:
        raise_exit_intent(
            f"Unsupported format: '{fmt}'",
            code=2,
            failure="format",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
            error_type=ErrorType.USER_INPUT,
        )

    for token in intent.tokens:
        if "\x00" in token:
            raise_exit_intent(
                "Embedded null byte in command path",
                code=3,
                failure="null_byte",
                command=command,
                fmt=intent.error_fmt,
                quiet=intent.quiet,
                include_runtime=intent.include_runtime,
                log_level=intent.log_level,
                error_type=ErrorType.ASCII,
            )
        try:
            token.encode("ascii")
        except UnicodeEncodeError:
            raise_exit_intent(
                f"Non-ASCII characters in command path: {token!r}",
                code=3,
                failure="ascii",
                command=command,
                fmt=intent.error_fmt,
                quiet=intent.quiet,
                include_runtime=intent.include_runtime,
                log_level=intent.log_level,
                error_type=ErrorType.ASCII,
            )

    if contains_non_ascii_env():
        raise_exit_intent(
            "Non-ASCII in environment",
            code=3,
            failure="ascii",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
            error_type=ErrorType.ASCII,
        )

    target = _find_target_command(ctx, intent.tokens)
    if not target:
        raise_exit_intent(
            f"No such command: {' '.join(intent.tokens)}",
            code=2,
            failure="not_found",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
            error_type=ErrorType.USER_INPUT,
        )

    target_cmd, target_ctx = target

    if intent.fmt_lower == _HUMAN:
        _emit_human_help(
            emit_output=emit_output,
            color=bool(resolve_click_color(quiet=intent.quiet, fmt=None)),
            help_text_provider=lambda: _get_formatted_help(target_cmd, target_ctx),
        )

    help_text = _capture_help_text(lambda: _get_formatted_help(target_cmd, target_ctx))
    try:
        payload = _build_help_payload(help_text, intent.include_runtime, started_at)
    except ValueError as exc:
        raise_exit_intent(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )

    output_format = (
        OutputFormat.YAML
        if intent.format_value == OutputFormat.YAML
        else OutputFormat.JSON
    )
    _emit_structured_help(
        command=command,
        payload=payload,
        output_format=output_format,
        pretty=intent.pretty,
        emit_output=emit_output,
    )


__all__ = ["ARGS", "help_app", "help_callback"]
