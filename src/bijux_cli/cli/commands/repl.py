# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the interactive Read-Eval-Print Loop (REPL) for the Bijux CLI.

This module provides a rich, interactive shell for executing Bijux CLI commands.
It enhances the user experience with features like persistent command history,
context-aware tab-completion, and a colorized prompt. Users can chain multiple
commands on a single line using semicolons. The REPL can also operate in a
non-interactive mode to process commands piped from stdin.

The REPL itself operates in a human-readable format. When executing commands,
it respects global flags like format or quiet for those specific invocations.

Exit Codes:
    * `0`: The REPL session was exited cleanly (e.g., via `exit`, `quit`,
      Ctrl+D, or a caught signal).
    * `2`: An invalid flag was provided to the `repl` command itself
      (e.g., an unsupported format).
"""

from __future__ import annotations

import sys

import typer

from bijux_cli.cli.core.command import validate_common_flags
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
from bijux_cli.cli.repl.execution import _run_piped as _exec_run_piped
from bijux_cli.cli.repl.ui import register_signal_handlers
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import current_execution_policy, resolve_exit_intent
from bijux_cli.core.runtime import AsyncTyper, run_command

repl_app = AsyncTyper(
    name="repl",
    help="Starts an interactive shell with history and tab-completion.",
    add_completion=False,
)


def _run_piped(repl_quiet: bool) -> None:
    """Run the REPL in non-interactive mode."""
    _exec_run_piped(repl_quiet)


async def _run_interactive() -> None:
    """Run the interactive REPL loop."""
    from bijux_cli.cli.repl.ui import _run_interactive as _ui_run_interactive

    await _ui_run_interactive()


def _run_repl_session(*, quiet: bool, stdin_isatty: bool) -> None:
    """Route to piped or interactive mode."""
    interactive = stdin_isatty and not quiet
    if not interactive:
        _run_piped(quiet)
        return
    run_command(_run_interactive)


@repl_app.callback(invoke_without_command=True)
def main(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("human", *OPT_FORMAT, help=HELP_FORMAT_HELP),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint for the `bijux repl` command.

    This function initializes the REPL environment. It validates flags, sets
    up signal handlers for clean shutdown, and dispatches to either the
    non-interactive (piped) mode or the interactive async prompt loop.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, forces non-interactive mode and suppresses
            prompts and command output.
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
    policy = current_execution_policy()
    effective_include_runtime = policy.include_runtime
    quiet = policy.quiet

    fmt_lower = fmt.strip().lower()
    format_value = None

    if fmt_lower != "human":
        format_value = validate_common_flags(
            fmt,
            command,
            policy.quiet,
            include_runtime=effective_include_runtime,
            log_level=policy.log_level,
        )
        intent = resolve_exit_intent(
            message="REPL only supports human format.",
            code=2,
            failure="format",
            command=command,
            fmt=format_value,
            quiet=policy.quiet,
            include_runtime=effective_include_runtime,
            error_type=ErrorType.USAGE,
            log_level=policy.log_level,
        )
        raise ExitIntentError(intent)

    register_signal_handlers()

    _run_repl_session(quiet=quiet, stdin_isatty=sys.stdin.isatty())


if __name__ == "__main__":
    repl_app()
