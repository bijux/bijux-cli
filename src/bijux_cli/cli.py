# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Constructs the main `Typer` application for the Bijux CLI.

This module serves as the primary builder for the entire CLI. It defines the
root `Typer` app, orchestrates the registration of all core commands and
the discovery of dynamic plugins, and sets the default behavior for when the
CLI is invoked without any command.
"""

from __future__ import annotations

import subprocess  # nosec B404
import sys

import typer
from typer import Context

from bijux_cli.commands import register_commands, register_dynamic_plugins


def maybe_default_to_repl(ctx: Context) -> None:
    """Invokes the `repl` command if no other subcommand is specified.

    This function is used as the root callback for the main `Typer` application.
    It checks if a subcommand was invoked and, if not, re-executes the CLI
    with the `repl` command.

    Args:
        ctx (Context): The Typer context, used to check for an invoked subcommand.

    Returns:
        None:
    """
    if ctx.invoked_subcommand is None:
        subprocess.call([sys.argv[0], "repl"])  # noqa: S603  # nosec B603


def build_app() -> typer.Typer:
    """Builds and configures the root `Typer` application.

    This factory function performs the main steps of assembling the CLI:
    1.  Creates the root `Typer` app instance.
    2.  Registers all core, built-in commands.
    3.  Discovers and registers all dynamic plugins.
    4.  Sets the default callback to launch the REPL.

    Returns:
        typer.Typer: The fully constructed `Typer` application.
    """
    app = typer.Typer(
        help="Bijux CLI – Lean, plug-in‑driven command‑line interface.",
        invoke_without_command=True,
        context_settings={
            "ignore_unknown_options": True,
            "allow_extra_args": True,
        },
    )
    register_commands(app)
    register_dynamic_plugins(app)
    app.callback(invoke_without_command=True)(maybe_default_to_repl)
    return app


app = build_app()

__all__ = ["build_app", "app"]
