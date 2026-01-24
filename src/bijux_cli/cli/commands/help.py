# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `help` command for the Bijux CLI.

This module provides a contextual help system that can generate and display
help text for any command or subcommand. It supports multiple output formats,
including human-readable text for interactive use and structured JSON or YAML
for automation and integration purposes. It also includes special logic to
suppress known noisy warnings from the plugin system during help generation.

Output Contract:
    * Human:      Standard CLI help text is printed to stdout.
    * JSON/YAML:  `{"help": str}`
    * Verbose:    Adds `{"python": str, "platform": str, "runtime_ms": int}`.
    * Error:      `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: Fatal or internal error.
    * `2`: CLI argument, flag, or "command not found" error.
    * `3`: ASCII or encoding error.
"""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
import platform as _platform
import sys
import time

import click
import typer

from bijux_cli.cli.color import resolve_click_color
from bijux_cli.cli.commands.payloads import HelpPayload
from bijux_cli.cli.core.constants import (
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
    OPT_VERBOSE,
)
from bijux_cli.cli.core.emit import (
    emit_and_exit,
    emit_text_and_exit,
)
from bijux_cli.cli.core.help_text import (
    HELP_FORMAT_HELP,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.core.output import current_execution_policy, emit_error_with_policy
from bijux_cli.cli.core.validation import (
    ascii_safe,
    contains_non_ascii_env,
    normalize_format,
    validate_common_flags,
)
from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat
from bijux_cli.core.precedence import ExecutionPolicy, LogPolicy
from bijux_cli.core.runtime import AsyncTyper

_HUMAN = "human"
_VALID_FORMATS = ("human", "json", "yaml")


@dataclass(frozen=True)
class HelpIntent:
    """Resolved intent for the help command."""

    tokens: list[str]
    fmt_lower: str
    format_value: OutputFormat | None
    error_fmt: OutputFormat
    include_runtime: bool
    pretty: bool
    quiet: bool
    log_policy: LogPolicy


def _build_help_intent(
    tokens: list[str],
    fmt: str,
    policy: ExecutionPolicy,
) -> HelpIntent:
    """Build a normalized help intent from raw CLI inputs."""
    fmt_lower = fmt.strip().lower()
    format_value = normalize_format(fmt)
    error_fmt = format_value or OutputFormat.JSON
    return HelpIntent(
        tokens=tokens,
        fmt_lower=fmt_lower,
        format_value=format_value,
        error_fmt=error_fmt,
        include_runtime=policy.include_runtime,
        pretty=policy.pretty,
        quiet=policy.quiet,
        log_policy=policy.log_policy,
    )


def _find_target_command(
    ctx: typer.Context, path: list[str]
) -> tuple[click.Command, click.Context] | None:
    """Locates the Click command and context for a given command path.

    Args:
        ctx (typer.Context): The Typer context object for the CLI.
        path (list[str]): A list of command and subcommand tokens.

    Returns:
        tuple[click.Command, click.Context] | None: A tuple containing the
            matched command and its context, or None if not found.
    """
    root_cmd: click.Command | None = ctx.parent.command if ctx.parent else None
    if not root_cmd:
        return None

    current_cmd: click.Command | None = root_cmd
    current_ctx = click.Context(root_cmd, info_name="bijux")

    for token in path:
        if not isinstance(current_cmd, click.Group):
            return None
        next_cmd = current_cmd.get_command(current_ctx, token)
        if not next_cmd:
            return None
        current_ctx = click.Context(next_cmd, info_name=token, parent=current_ctx)
        current_cmd = next_cmd

    assert current_cmd is not None  # noqa: S101 # nosec: B101
    return current_cmd, current_ctx


def _get_formatted_help(cmd: click.Command, ctx: click.Context) -> str:
    """Gets and formats the help text for a command.

    This helper ensures that the short help option '-h' is included in the
    final help text if it was defined in the command's context settings.

    Args:
        cmd (click.Command): The Click command object.
        ctx (click.Context): The Click context for the command.

    Returns:
        str: The formatted help text.
    """
    help_text = cmd.get_help(ctx)
    if (
        hasattr(cmd, "context_settings")
        and cmd.context_settings
        and "-h" in cmd.context_settings.get("help_option_names", [])
        and "-h, --help" not in help_text
    ):
        help_text = help_text.replace("--help", "-h, --help")
    return help_text


def _build_help_payload(
    help_text: str, include_runtime: bool, started_at: float
) -> HelpPayload:
    """Builds a structured help payload for JSON/YAML output.

    Args:
        help_text (str): The CLI help text to be included in the payload.
        include_runtime (bool): If True, adds Python, platform, and runtime
            metadata to the payload.
        started_at (float): The start time from `time.perf_counter()` to use
            for calculating the runtime duration.

    Returns:
        HelpPayload: A payload containing help text and optional runtime fields.
    """
    payload = HelpPayload(help=help_text)
    if include_runtime:
        return HelpPayload(
            help=payload.help,
            python=ascii_safe(sys.version.split()[0], "python_version"),
            platform=ascii_safe(_platform.platform(), "platform"),
            runtime_ms=int((time.perf_counter() - started_at) * 1_000),
        )
    return payload


def _emit_human_help(
    *,
    quiet: bool,
    color: bool | None,
    help_text_provider: Callable[[], str],
) -> None:
    """Emit human help output without building text in quiet mode."""
    if quiet:
        from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

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
    emit_text_and_exit(help_text_provider(), color=color, exit_code=0)


typer.core.rich = None  # type: ignore[attr-defined,assignment]

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
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
    fmt: str = typer.Option(_HUMAN, *OPT_FORMAT, help=HELP_FORMAT_HELP),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux help` command.

    This function orchestrates the entire help generation process. It parses the
    target command path, finds the corresponding command object, performs ASCII
    and format validation, and emits the help text in the specified format.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        command_path (list[str] | None): A list of tokens representing the path
            to the target command (e.g., `["config", "get"]`).
        quiet (bool): If True, suppresses all output. The exit code is the
            primary indicator of outcome.
        verbose (bool): If True, includes Python and platform details in
            structured output formats.
        fmt (str): The output format: "human", "json", or "yaml".
        pretty (bool): If True, pretty-prints structured output.
        log_level (str): Logging level for diagnostics.
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant exit code and payload
            upon completion or error.
    """
    started_at = time.perf_counter()

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
            policy = current_execution_policy()
            if policy.quiet:
                from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

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
            emit_text_and_exit(
                help_text,
                color=resolve_click_color(quiet=policy.quiet, fmt=None),
                exit_code=0,
            )
        else:
            from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

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
    _ = (quiet, verbose, log_level, pretty, fmt)
    policy = current_execution_policy()
    intent = _build_help_intent(tokens, fmt, policy)

    if intent.fmt_lower != "human":
        validate_common_flags(
            intent.format_value or OutputFormat.JSON,
            command,
            intent.quiet,
            include_runtime=intent.include_runtime,
        )

    if intent.fmt_lower not in _VALID_FORMATS:
        emit_error_with_policy(
            f"Unsupported format: '{fmt}'",
            code=2,
            failure="format",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_policy=intent.log_policy,
            error_type=ErrorType.USER_INPUT,
        )

    for token in intent.tokens:
        if "\x00" in token:
            emit_error_with_policy(
                "Embedded null byte in command path",
                code=3,
                failure="null_byte",
                command=command,
                fmt=intent.error_fmt,
                quiet=intent.quiet,
                include_runtime=intent.include_runtime,
                log_policy=intent.log_policy,
                error_type=ErrorType.ASCII,
            )
        try:
            token.encode("ascii")
        except UnicodeEncodeError:
            emit_error_with_policy(
                f"Non-ASCII characters in command path: {token!r}",
                code=3,
                failure="ascii",
                command=command,
                fmt=intent.error_fmt,
                quiet=intent.quiet,
                include_runtime=intent.include_runtime,
                log_policy=intent.log_policy,
                error_type=ErrorType.ASCII,
            )

    if contains_non_ascii_env():
        emit_error_with_policy(
            "Non-ASCII in environment",
            code=3,
            failure="ascii",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_policy=intent.log_policy,
            error_type=ErrorType.ASCII,
        )

    target = _find_target_command(ctx, intent.tokens)
    if not target:
        emit_error_with_policy(
            f"No such command: {' '.join(intent.tokens)}",
            code=2,
            failure="not_found",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_policy=intent.log_policy,
            error_type=ErrorType.USER_INPUT,
        )

    target_cmd, target_ctx = target

    if intent.fmt_lower == _HUMAN:
        _emit_human_help(
            quiet=intent.quiet,
            color=resolve_click_color(quiet=intent.quiet, fmt=None),
            help_text_provider=lambda: _get_formatted_help(target_cmd, target_ctx),
        )

    help_text = _get_formatted_help(target_cmd, target_ctx)

    try:
        payload = _build_help_payload(help_text, intent.include_runtime, started_at)
    except ValueError as exc:
        emit_error_with_policy(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=intent.error_fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_policy=intent.log_policy,
        )

    output_format = (
        OutputFormat.YAML
        if intent.format_value == OutputFormat.YAML
        else OutputFormat.JSON
    )
    if intent.quiet:
        from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

        raise ExitIntentError(
            ExitIntent(
                code=ExitCode.SUCCESS,
                stream=None,
                payload=None,
                fmt=output_format,
                pretty=intent.pretty,
                show_traceback=False,
            )
        )
    emit_and_exit(
        payload=payload,
        fmt=output_format,
        effective_pretty=intent.pretty,
        verbose=policy.verbose,
        command=command,
        exit_code=0,
    )
