# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Constructs the command structure for the Bijux CLI application."""

from __future__ import annotations

import logging

from typer import Typer

from bijux_cli.cli.commands.config import config_app
from bijux_cli.cli.commands.dev import dev_app
from bijux_cli.cli.commands.diagnostics.audit import audit_app
from bijux_cli.cli.commands.diagnostics.docs_command import docs_app
from bijux_cli.cli.commands.diagnostics.doctor import doctor_app
from bijux_cli.cli.commands.help_command import help_app
from bijux_cli.cli.commands.history import history_app
from bijux_cli.cli.commands.memory import memory_app
from bijux_cli.cli.commands.repl import repl_app
from bijux_cli.cli.commands.sleep import sleep_app
from bijux_cli.cli.commands.status import status_app
from bijux_cli.cli.commands.version import version_app
from bijux_cli.cli.plugins.commands import plugins_app

logger = logging.getLogger(__name__)

_CORE_COMMANDS = {
    "audit": audit_app,
    "config": config_app,
    "dev": dev_app,
    "docs": docs_app,
    "doctor": doctor_app,
    "help": help_app,
    "history": history_app,
    "memory": memory_app,
    "plugins": plugins_app,
    "repl": repl_app,
    "status": status_app,
    "version": version_app,
    "sleep": sleep_app,
}
_REGISTERED_COMMANDS: set[str] = set(_CORE_COMMANDS.keys())


def register_commands(app: Typer) -> list[str]:
    """Registers all core, built-in commands with the main Typer application."""
    _REGISTERED_COMMANDS.clear()
    _REGISTERED_COMMANDS.update(_CORE_COMMANDS.keys())
    for name, cmd in sorted(_CORE_COMMANDS.items()):
        app.add_typer(cmd, name=name, invoke_without_command=True)
        _REGISTERED_COMMANDS.add(name)
    return sorted(_CORE_COMMANDS.keys())


def register_dynamic_plugins(app: Typer) -> None:
    """Discovers and registers all third-party plugins."""
    import inspect

    from bijux_cli.plugins.loader import activate_plugin
    from bijux_cli.plugins.metadata import discover_plugins

    try:
        if "strict" in inspect.signature(discover_plugins).parameters:
            plugins = discover_plugins(strict=False)
        else:
            plugins = discover_plugins()
    except Exception as exc:
        logger.warning("Plugin discovery failed: %s", exc)
        return

    for meta in plugins:
        if meta.name in _REGISTERED_COMMANDS:
            raise RuntimeError(f"Plugin name collision: {meta.name!r}")
        try:
            app.add_typer(activate_plugin(meta), name=meta.name)
        except Exception as exc:
            logger.warning("Plugin %s failed to register: %s", meta.name, exc)
            continue
        _REGISTERED_COMMANDS.add(meta.name)


def list_registered_command_names() -> list[str]:
    """Returns a list of all registered command names."""
    return sorted(_REGISTERED_COMMANDS)


__all__ = [
    "register_commands",
    "register_dynamic_plugins",
    "list_registered_command_names",
]
