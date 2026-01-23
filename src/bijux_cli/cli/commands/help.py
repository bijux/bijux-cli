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

from collections.abc import Mapping
import platform as _platform
import sys
import time
from typing import Any

import click
import typer

from bijux_cli.cli.color import resolve_click_color
from bijux_cli.cli.constants import (
    HELP_FORMAT_HELP,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.emit import emit_and_exit, emit_error_and_exit
from bijux_cli.cli.output import get_execution_policy
from bijux_cli.cli.validation import (
    ascii_safe,
    contains_non_ascii_env,
    normalize_format,
    validate_common_flags,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import LogLevel, OutputFormat
from bijux_cli.core.runtime import AsyncTyper
from bijux_cli.infra.contracts import Emitter

_HUMAN = "human"
_VALID_FORMATS = ("human", "json", "yaml")


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
) -> Mapping[str, Any]:
    """Builds a structured help payload for JSON/YAML output.

    Args:
        help_text (str): The CLI help text to be included in the payload.
        include_runtime (bool): If True, adds Python, platform, and runtime
            metadata to the payload.
        started_at (float): The start time from `time.perf_counter()` to use
            for calculating the runtime duration.

    Returns:
        Mapping[str, Any]: A payload containing help text and optional runtime
            fields.
    """
    payload: dict[str, Any] = {"help": help_text}
    if include_runtime:
        payload["python"] = ascii_safe(sys.version.split()[0], "python_version")
        payload["platform"] = ascii_safe(_platform.platform(), "platform")
        payload["runtime_ms"] = int((time.perf_counter() - started_at) * 1_000)
    return payload


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
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option(_HUMAN, "-f", "--format", help=HELP_FORMAT_HELP),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
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
        debug (bool): If True, enables debug diagnostics, implying `verbose`
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
        known_flags_with_args = {"-f", "--format"}
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
            policy = get_execution_policy()
            typer.echo(
                help_text,
                color=resolve_click_color(quiet=policy.quiet, fmt=None),
            )
        raise typer.Exit(0)

    tokens = command_path or []
    command = "help"
    _ = (quiet, verbose, log_level, pretty, fmt)
    policy = get_execution_policy()
    effective_include_runtime = policy.include_runtime
    effective_pretty = policy.pretty
    fmt_lower = fmt.strip().lower()
    format_value = normalize_format(fmt)
    error_fmt = format_value or OutputFormat.JSON

    if policy.quiet:
        if fmt_lower not in _VALID_FORMATS:
            raise SystemExit(2)

        for token in tokens:
            if "\x00" in token:
                raise SystemExit(3)
            try:
                token.encode("ascii")
            except UnicodeEncodeError as err:
                raise SystemExit(3) from err

        if contains_non_ascii_env():
            raise SystemExit(3)

        if not _find_target_command(ctx, tokens):
            raise SystemExit(2)

        raise SystemExit(0)

    if fmt_lower != "human":
        validate_common_flags(
            format_value or OutputFormat.JSON,
            command,
            policy.quiet,
            include_runtime=effective_include_runtime,
        )

    if fmt_lower not in _VALID_FORMATS:
        emit_error_and_exit(
            f"Unsupported format: '{fmt}'",
            code=2,
            failure="format",
            command=command,
            fmt=error_fmt,
            quiet=policy.quiet,
            include_runtime=effective_include_runtime,
            debug=(policy.log_level == LogLevel.DEBUG),
        )

    for token in tokens:
        if "\x00" in token:
            emit_error_and_exit(
                "Embedded null byte in command path",
                code=3,
                failure="null_byte",
                command=command,
                fmt=error_fmt,
                quiet=policy.quiet,
                include_runtime=effective_include_runtime,
                debug=(policy.log_level == LogLevel.DEBUG),
            )
        try:
            token.encode("ascii")
        except UnicodeEncodeError:
            emit_error_and_exit(
                f"Non-ASCII characters in command path: {token!r}",
                code=3,
                failure="ascii",
                command=command,
                fmt=error_fmt,
                quiet=policy.quiet,
                include_runtime=effective_include_runtime,
                debug=(policy.log_level == LogLevel.DEBUG),
            )

    if contains_non_ascii_env():
        emit_error_and_exit(
            "Non-ASCII in environment",
            code=3,
            failure="ascii",
            command=command,
            fmt=error_fmt,
            quiet=policy.quiet,
            include_runtime=effective_include_runtime,
            debug=(policy.log_level == LogLevel.DEBUG),
        )

    target = _find_target_command(ctx, tokens)
    if not target:
        emit_error_and_exit(
            f"No such command: {' '.join(tokens)}",
            code=2,
            failure="not_found",
            command=command,
            fmt=error_fmt,
            quiet=policy.quiet,
            include_runtime=effective_include_runtime,
            debug=(policy.log_level == LogLevel.DEBUG),
        )

    DIContainer.current().resolve(Emitter)
    target_cmd, target_ctx = target
    help_text = _get_formatted_help(target_cmd, target_ctx)

    if fmt_lower == _HUMAN:
        typer.echo(
            help_text,
            color=resolve_click_color(quiet=policy.quiet, fmt=None),
        )
        raise typer.Exit(0)

    try:
        payload = _build_help_payload(help_text, effective_include_runtime, started_at)
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=error_fmt,
            quiet=policy.quiet,
            include_runtime=effective_include_runtime,
            debug=(policy.log_level == LogLevel.DEBUG),
        )

    output_format = (
        OutputFormat.YAML if format_value == OutputFormat.YAML else OutputFormat.JSON
    )
    emit_and_exit(
        payload=payload,
        fmt=output_format,
        effective_pretty=effective_pretty,
        verbose=policy.verbose,
        debug=(policy.log_level == LogLevel.DEBUG),
        quiet=policy.quiet,
        command=command,
        exit_code=0,
    )
