# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the `dev` command group for the Bijux CLI.

This module serves as the central entry point for all developer-focused tools
and diagnostics. It aggregates the various subcommands into a single `Typer`
application, creating the `bijux dev` command hierarchy.

The `dev` command, when run without a subcommand, provides a simple status
confirmation.

The available subcommands are:
    * `di`: Displays the dependency injection (DI) container graph.
    * `list-plugins`: Lists all installed CLI plugins.

Exit codes for the subcommands generally follow this pattern:
    * `0`: Success.
    * `1`: An internal or unexpected error occurred.
    * `2`: An invalid argument was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected.
"""

from __future__ import annotations

import typer

from bijux_cli.commands.dev.di import dev_di_graph
from bijux_cli.commands.dev.list_plugins import dev_list_plugins
from bijux_cli.commands.dev.service import dev

typer.core.rich = None  # type: ignore[attr-defined,assignment]

dev_app = typer.Typer(  # pytype: skip-file
    name="dev",
    help="Developer tools and diagnostics.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

dev_app.callback(invoke_without_command=True)(dev)
dev_app.command("di")(dev_di_graph)
dev_app.command("list-plugins")(dev_list_plugins)

__all__ = ["dev_app"]
