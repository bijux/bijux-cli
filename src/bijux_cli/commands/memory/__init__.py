# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Defines the `memory` command group for the Bijux CLI.

This module serves as the central entry point for managing the CLI's transient,
in-memory data store. It aggregates the various subcommands into a single
`Typer` application, creating the `bijux memory` command hierarchy. The data
in this store persists only for the lifetime of the application's parent process.

The `memory` command, when run without a subcommand, provides a brief summary
of the memory store's state.

The available subcommands are:
    * `set`: Sets a key-value pair.
    * `get`: Retrieves the value for a specific key.
    * `delete`: Removes a key-value pair.
    * `list`: Lists all defined keys.
    * `clear`: Removes all key-value pairs from the memory store.

Exit codes for the subcommands generally follow this pattern:
    * `0`: Success.
    * `1`: An internal or unexpected error occurred (e.g., service unavailable).
    * `2`: An invalid argument was provided (e.g., bad format, invalid key).
    * `3`: An ASCII or encoding error was detected.
"""

from __future__ import annotations

import typer

from bijux_cli.commands.memory.clear import clear_memory
from bijux_cli.commands.memory.delete import delete_memory
from bijux_cli.commands.memory.get import get_memory
from bijux_cli.commands.memory.list import list_memory
from bijux_cli.commands.memory.service import memory
from bijux_cli.commands.memory.set import set_memory

typer.core.rich = None  # type: ignore[attr-defined,assignment]

memory_app = typer.Typer(  # pytype: skip-file
    name="memory",
    help="Manage CLI memory.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

memory_app.callback(invoke_without_command=True)(memory)

memory_app.command("set")(set_memory)
memory_app.command("get")(get_memory)
memory_app.command("delete")(delete_memory)
memory_app.command("clear")(clear_memory)
memory_app.command("list")(list_memory)

__all__ = [
    "memory_app",
]
