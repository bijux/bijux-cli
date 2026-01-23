# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the interactive Read-Eval-Print Loop (REPL) for the Bijux CLI.

This module provides a rich, interactive shell for executing Bijux CLI commands.
It enhances the user experience with features like persistent command history,
context-aware tab-completion, and a colorized prompt. Users can chain multiple
commands on a single line using semicolons. The REPL can also operate in a
non-interactive mode to process commands piped from stdin.

The REPL itself operates in a human-readable format. When executing commands,
it respects global flags like `--format` or `--quiet` for those specific
invocations.

Exit Codes:
    * `0`: The REPL session was exited cleanly (e.g., via `exit`, `quit`,
      Ctrl+D, or a caught signal).
    * `2`: An invalid flag was provided to the `repl` command itself
      (e.g., `--format=json`).
"""

from __future__ import annotations

import sys

import typer

from bijux_cli.cli.constants import (
    HELP_FORMAT_HELP,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.emit import emit_error_and_exit
from bijux_cli.cli.repl.execution import _run_piped
from bijux_cli.cli.repl.ui import (
    _run_interactive,
    register_signal_handlers,
)
from bijux_cli.cli.validation import validate_common_flags
from bijux_cli.core.enums import LogLevel
from bijux_cli.core.runtime import AsyncTyper, run_command

repl_app = AsyncTyper(
    name="repl",
    help="Starts an interactive shell with history and tab-completion.",
    add_completion=False,
)


@repl_app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("human", "-f", "--format", help=HELP_FORMAT_HELP),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint for the `bijux repl` command.

    This function initializes the REPL environment. It validates flags, sets
    up signal handlers for clean shutdown, and dispatches to either the
    non-interactive (piped) mode or the interactive async prompt loop.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, forces non-interactive mode and suppresses
            prompts and command output.
        verbose (bool): If True, enables verbose output for subcommands.
        fmt (str): The desired output format. Only "human" is supported for
            the REPL itself.
        pretty (bool): If True, enables pretty-printing for subcommands.
        log_level (str): The requested logging level for subcommands.

    Returns:
        None:
    """
    if ctx.invoked_subcommand:
        return

    command = "repl"
    from bijux_cli.cli.output import get_execution_policy

    _ = (quiet, verbose, log_level, pretty, fmt)
    policy = get_execution_policy()
    effective_include_runtime = policy.include_runtime
    quiet = policy.quiet
    verbose = policy.verbose
    pretty = policy.pretty

    fmt_lower = fmt.strip().lower()
    format_value = None

    if fmt_lower != "human":
        format_value = validate_common_flags(
            fmt,
            command,
            policy.quiet,
            include_runtime=effective_include_runtime,
        )
        emit_error_and_exit(
            "REPL only supports human format.",
            code=2,
            failure="format",
            command=command,
            fmt=format_value,
            quiet=policy.quiet,
            include_runtime=effective_include_runtime,
            debug=(policy.log_level == LogLevel.DEBUG),
        )

    register_signal_handlers()

    if quiet or not sys.stdin.isatty():
        _run_piped(quiet)
    else:
        run_command(_run_interactive)


if __name__ == "__main__":
    repl_app()
