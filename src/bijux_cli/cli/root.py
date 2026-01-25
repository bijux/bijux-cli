# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Build the Bijux CLI root Typer application and register plugins.

This module assembles the root Typer app, registers core commands, and discovers
external plugins via entry points:

* ``bijux.commands``: each entry must be a ``Typer`` app mounted under its
  entry-point name.
* ``bijux_cli.plugins``: flexible plugins that may:
  - return a ``Typer`` app,
  - be a callable factory/class (instantiated with no arguments),
  - expose ``registered_groups: dict[str, Typer]``,
  - expose ``register(app: Typer)`` to register commands/groups.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping
import logging
import subprocess  # noqa: S603  # nosec B404 - controlled internal call
import sys
from typing import Any

import typer
from typer import Context

from bijux_cli.cli.color import resolve_click_color
from bijux_cli.cli.commands import register_commands, register_dynamic_plugins
from bijux_cli.core.intent import parse_global_config
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.core.runtime import AsyncTyper

logger = logging.getLogger(__name__)
if not logger.handlers:
    logger.addHandler(logging.NullHandler())


def _collect_names(container: Mapping[Any, Any] | Iterable[Any]) -> list[str]:
    """Collect command/group names from a Typer registry-like container.

    Args:
        container: A list-like or dict-like container holding Typer objects.

    Returns:
        A list of registered names.
    """
    items: Iterable[Any] = (
        container.values() if isinstance(container, Mapping) else container
    )

    names: list[str] = []
    for obj in items:
        name = getattr(obj, "name", None)
        if isinstance(name, str) and name:
            names.append(name)
    return names


def _existing_top_level_names(app: typer.Typer) -> set[str]:
    """Return the set of names already registered at the top level.

    Args:
        app: The root Typer application.

    Returns:
        A set of names for existing groups and commands.
    """
    groups = _collect_names(getattr(app, "registered_groups", []) or [])
    commands = _collect_names(getattr(app, "registered_commands", []) or [])
    return set(groups) | set(commands)


def register_entrypoint_plugins(app: AsyncTyper) -> None:
    """Discover and register plugins exposed via entry points.

    Args:
        app: The root Typer application.
    """
    register_dynamic_plugins(app)


def maybe_default_to_repl(ctx: Context) -> None:
    """Launch the REPL when invoked with no args; otherwise show help on error.

    If no subcommand is chosen and no extra CLI arguments are provided, the
    function re-invokes the executable with the ``repl`` command. If arguments
    are present but no subcommand is resolved, it prints help and exits with
    code 2.

    Args:
        ctx: The Typer context.
    """
    if ctx.invoked_subcommand is None and len(sys.argv) == 1:
        subprocess.call(  # noqa: S603  # nosec B603 - controlled argv
            [sys.argv[0], "repl"]
        )
    elif ctx.invoked_subcommand is None:
        policy = current_execution_policy()
        typer.echo(
            ctx.get_help(),
            color=resolve_click_color(quiet=policy.quiet, fmt=None),
        )
        raise typer.Exit(code=2)


def _log_registered(app: typer.Typer) -> None:
    """Log the names of registered core commands and groups at debug level.

    Args:
        app: The root Typer application.
    """
    cmds = _collect_names(getattr(app, "registered_commands", []) or [])
    grps = _collect_names(getattr(app, "registered_groups", []) or [])
    logger.debug("Core commands registered: %s", cmds)
    logger.debug("Core groups registered: %s", grps)


def build_app(*, load_plugins: bool = True) -> AsyncTyper:
    """Construct the root Typer application.

    Returns:
        The fully assembled Typer application with core and plugin commands.
    """
    app = AsyncTyper(
        help="Bijux CLI – Lean, plug-in-driven command-line interface.",
        invoke_without_command=True,
    )
    register_commands(app)
    _log_registered(app)
    if load_plugins:
        register_dynamic_plugins(app)
    app.callback(invoke_without_command=True)(maybe_default_to_repl)
    return app


app = build_app()


__all__ = [
    "app",
    "build_app",
    "parse_global_config",
    "register_entrypoint_plugins",
]
