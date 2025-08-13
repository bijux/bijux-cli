# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the `history` command group for the Bijux CLI.

This module serves as the central entry point for all command history
management functionality. It aggregates the various subcommands and the root
history command into a single `Typer` application.

The `history` command, when run without a subcommand, lists, filters, sorts,
imports, or exports history entries depending on the flags provided.

The available subcommands are:
    * `clear`: Erases all stored command history.

Exit codes for the subcommands generally follow this pattern:
    * `0`: Success.
    * `1`: An internal or unexpected error occurred.
    * `2`: An invalid argument was provided (e.g., bad format, I/O error).
    * `3`: An ASCII or encoding error was detected.
"""

from __future__ import annotations

import typer

from bijux_cli.commands.history.clear import clear_history
from bijux_cli.commands.history.service import history

typer.core.rich = None  # type: ignore[attr-defined,assignment]

history_app = typer.Typer(  # pytype: skip-file
    name="history",
    help="Inspect or manage command history.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

history_app.callback(invoke_without_command=True)(history)
history_app.command("clear")(clear_history)

__all__ = ["history_app"]
