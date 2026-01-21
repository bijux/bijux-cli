# SPDX-License-Identifier: MIT
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
import importlib.metadata as md
import logging
import subprocess  # noqa: S603
import sys
from typing import Any

import typer
from typer import Context, Typer

from bijux_cli.commands import register_commands, register_dynamic_plugins

logger = logging.getLogger(__name__)
if not logger.handlers:
    logger.addHandler(logging.NullHandler())


def _iter_entry_points(group: str) -> Iterable[md.EntryPoint]:
    """Return entry points for a given group.

    Args:
        group: The entry-point group name.

    Returns:
        An iterable of entry points.
    """
    return md.entry_points(group=group)


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


def _existing_top_level_names(app: Typer) -> set[str]:
    """Return the set of names already registered at the top level.

    Args:
        app: The root Typer application.

    Returns:
        A set of names for existing groups and commands.
    """
    groups = _collect_names(getattr(app, "registered_groups", []) or [])
    commands = _collect_names(getattr(app, "registered_commands", []) or [])
    return set(groups) | set(commands)


def _safe_add_typer(app: Typer, sub: Typer, name: str, seen: set[str]) -> None:
    """Add a Typer sub-app if the name is not taken.

    Args:
        app: The root Typer application.
        sub: The Typer sub-application to add.
        name: The mount name.
        seen: The set of already used names.
    """
    if name in seen:
        logger.debug("Skipped plugin group '%s' (name already taken)", name)
        return
    app.add_typer(sub, name=name)
    seen.add(name)
    logger.debug("Registered plugin group: %s", name)


def register_entrypoint_plugins(app: Typer) -> None:
    """Discover and register plugins exposed via entry points.

    Args:
        app: The root Typer application.
    """
    seen: set[str] = _existing_top_level_names(app)

    for ep in _iter_entry_points("bijux.commands"):
        try:
            obj = ep.load()
            if isinstance(obj, Typer):
                _safe_add_typer(app, obj, ep.name, seen)
            else:
                logger.debug(
                    "Entry point '%s' in 'bijux.commands' is not a Typer app", ep.name
                )
        except Exception as exc:
            logger.debug("Failed to load entry point %s: %s", ep.name, exc)

    for ep in _iter_entry_points("bijux_cli.plugins"):
        try:
            plugin = ep.load()
            if isinstance(plugin, Typer):
                _safe_add_typer(app, plugin, ep.name, seen)
                continue
            if callable(plugin):
                try:
                    plugin = plugin()
                except Exception as inst_exc:
                    logger.debug(
                        "Failed to instantiate plugin %s: %s", ep.name, inst_exc
                    )
                    continue
            groups = getattr(plugin, "registered_groups", None)
            if isinstance(groups, dict):
                for name, sub in groups.items():
                    if isinstance(sub, Typer):
                        _safe_add_typer(app, sub, name, seen)
            register_hook = getattr(plugin, "register", None)
            if callable(register_hook):
                try:
                    register_hook(app)
                except Exception as hook_exc:
                    logger.debug(
                        "Plugin '%s' register(app) failed: %s", ep.name, hook_exc
                    )
            if not isinstance(plugin, Typer):
                maybe_typer = getattr(plugin, "app", None)
                if isinstance(maybe_typer, Typer):
                    _safe_add_typer(app, maybe_typer, ep.name, seen)
        except Exception as exc:
            logger.debug("Failed to load plugin entry point %s: %s", ep.name, exc)


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
        subprocess.call([sys.argv[0], "repl"])  # noqa: S603
    elif ctx.invoked_subcommand is None:
        typer.echo(ctx.get_help())
        raise typer.Exit(code=2)


def _log_registered(app: Typer) -> None:
    """Log the names of registered core commands and groups at debug level.

    Args:
        app: The root Typer application.
    """
    cmds = _collect_names(getattr(app, "registered_commands", []) or [])
    grps = _collect_names(getattr(app, "registered_groups", []) or [])
    logger.debug("Core commands registered: %s", cmds)
    logger.debug("Core groups registered: %s", grps)


def build_app() -> Typer:
    """Construct the root Typer application.

    Returns:
        The fully assembled Typer application with core and plugin commands.
    """
    app = typer.Typer(
        help="Bijux CLI – Lean, plug-in-driven command-line interface.",
        invoke_without_command=True,
    )
    register_commands(app)
    _log_registered(app)
    register_dynamic_plugins(app)
    register_entrypoint_plugins(app)
    app.callback(invoke_without_command=True)(maybe_default_to_repl)
    return app


app = build_app()

__all__ = ["build_app", "app", "register_entrypoint_plugins"]
