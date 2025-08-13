# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the `plugins` command group for the Bijux CLI.

This module serves as the central entry point for all plugin management
functionality. It aggregates the various subcommands into a single `Typer`
application, creating the `bijux plugins` command hierarchy.

This command group does not have a default action and requires a subcommand
to be specified.

The available subcommands are:
    * `scaffold`: Creates a new plugin project from a `cookiecutter` template.
    * `install`: Installs a plugin from a local source directory.
    * `uninstall`: Removes an installed plugin.
    * `list`: Lists all installed plugins.
    * `info`: Displays detailed metadata for a specific plugin.
    * `check`: Runs a health check on an installed plugin.

Exit codes for the subcommands generally follow this pattern:
    * `0`: Success.
    * `1`: An internal error or command failure occurred.
    * `2`: An invalid argument or format was provided.
    * `3`: An ASCII or encoding error was detected.
"""

from __future__ import annotations

import typer

from bijux_cli.commands.plugins.check import check_plugin
from bijux_cli.commands.plugins.info import info_plugin
from bijux_cli.commands.plugins.install import install_plugin
from bijux_cli.commands.plugins.list import list_plugin
from bijux_cli.commands.plugins.scaffold import scaffold_plugin
from bijux_cli.commands.plugins.uninstall import uninstall_plugin

typer.core.rich = None  # type: ignore[attr-defined,assignment]

plugins_app = typer.Typer(  # pytype: skip-file
    name="plugins",
    help="Manage Bijux CLI plugins",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

plugins_app.command("scaffold")(scaffold_plugin)
plugins_app.command("install")(install_plugin)
plugins_app.command("uninstall")(uninstall_plugin)
plugins_app.command("list")(list_plugin)
plugins_app.command("info")(info_plugin)
plugins_app.command("check")(check_plugin)


__all__ = [
    "plugins_app",
]
