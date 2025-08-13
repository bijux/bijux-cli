# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the `config` command group for the Bijux CLI.

This module serves as the central entry point for all configuration management
functionality. It aggregates the various subcommands into a single `Typer`
application, creating the `bijux config` command hierarchy.

The `config` command, when run without a subcommand, lists all current
key-value pairs.

The available subcommands are:
    * `set`: Sets a key-value pair.
    * `get`: Retrieves the value for a specific key.
    * `unset`: Removes a key-value pair.
    * `list`: Lists all defined configuration keys.
    * `clear`: Removes all key-value pairs from the configuration.
    * `load`: Replaces the current configuration with one from a file.
    * `reload`: Forces a reload of the configuration from its source file.
    * `export`: Writes the current configuration to a file or stdout.

Exit codes for the subcommands generally follow this pattern:
    * `0`: Success.
    * `1`: An internal or unexpected error occurred.
    * `2`: An invalid argument was provided (e.g., bad format, key not found).
    * `3`: An ASCII or encoding error was detected.
"""

from __future__ import annotations

import typer

from bijux_cli.commands.config.clear import clear_config
from bijux_cli.commands.config.export import export_config
from bijux_cli.commands.config.get import get_config
from bijux_cli.commands.config.list_cmd import list_config
from bijux_cli.commands.config.load import load_config
from bijux_cli.commands.config.reload import reload_config
from bijux_cli.commands.config.service import config
from bijux_cli.commands.config.set import set_config
from bijux_cli.commands.config.unset import unset_config

typer.core.rich = None  # type: ignore[attr-defined,assignment]

config_app = typer.Typer(  # pytype: skip-file
    name="config",
    help="Manage CLI configuration.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

config_app.callback(invoke_without_command=True)(config)

config_app.command("clear")(clear_config)
config_app.command("export")(export_config)
config_app.command("get")(get_config)
config_app.command("list")(list_config)
config_app.command("load")(load_config)
config_app.command("reload")(reload_config)
config_app.command("set")(set_config)
config_app.command("unset")(unset_config)


@config_app.command("import", hidden=True)
def import_config(*args, **kwargs) -> None:  # type: ignore
    """Provides a backward-compatibility alias for the `config load` command.

    This command is hidden from the main help text and delegates directly to
    `load_config`, forwarding all arguments.

    Args:
        *args: Positional arguments to forward to `load_config`.
        **kwargs: Keyword arguments to forward to `load_config`.

    Returns:
        None:
    """
    return load_config(*args, **kwargs)


__all__ = [
    "config_app",
]
