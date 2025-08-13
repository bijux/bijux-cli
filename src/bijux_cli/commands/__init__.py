# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Constructs the command structure for the Bijux CLI application.

This module is responsible for assembling the entire CLI by registering all
core command groups and dynamically discovering and loading external plugins.
It provides the central mechanism that makes the CLI's command system modular
and extensible.

The primary functions are:
* `register_commands`: Attaches all built-in command `Typer` applications
    (e.g., `config_app`, `plugins_app`) to the main root application.
* `register_dynamic_plugins`: Scans for plugins from package entry points
    and the local plugins directory, loading and attaching them to the root
    application. Errors during this process are logged as warnings.
* `list_registered_command_names`: Provides a way to retrieve the names of all
    successfully registered commands, including dynamic plugins.
"""

from __future__ import annotations

import logging

from typer import Typer

from bijux_cli.commands.audit import audit_app
from bijux_cli.commands.config import config_app
from bijux_cli.commands.dev import dev_app
from bijux_cli.commands.docs import docs_app
from bijux_cli.commands.doctor import doctor_app
from bijux_cli.commands.help import help_app
from bijux_cli.commands.history import history_app
from bijux_cli.commands.memory import memory_app
from bijux_cli.commands.plugins import plugins_app
from bijux_cli.commands.repl import repl_app
from bijux_cli.commands.sleep import sleep_app
from bijux_cli.commands.status import status_app
from bijux_cli.commands.version import version_app

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
    """Registers all core, built-in commands with the main Typer application.

    Args:
        app (Typer): The main Typer application to which commands will be added.

    Returns:
        list[str]: An alphabetically sorted list of the names of the registered
            core commands.
    """
    for name, cmd in sorted(_CORE_COMMANDS.items()):
        app.add_typer(cmd, name=name, invoke_without_command=True)
        _REGISTERED_COMMANDS.add(name)
    return sorted(_CORE_COMMANDS.keys())


def register_dynamic_plugins(app: Typer) -> None:
    """Discovers and registers all third-party plugins.

    This function scans for plugins from two sources:
    1.  Python package entry points registered under the `bijux_cli.plugins` group.
    2.  Subdirectories within the local plugins folder that contain a `plugin.py`.

    For each discovered plugin, this function expects the loaded module to expose
    either a callable `cli()` that returns a `Typer` app or an `app` attribute
    that is a `Typer` instance. All discovery and loading errors are logged
    and suppressed to prevent a single faulty plugin from crashing the CLI.

    Args:
        app (Typer): The root Typer application to which discovered plugin
            apps will be attached.

    Returns:
        None:
    """
    import importlib.util
    import sys

    try:
        import importlib.metadata

        eps = importlib.metadata.entry_points()
        for ep in eps.select(group="bijux_cli.plugins"):
            try:
                plugin_app = ep.load()
                app.add_typer(plugin_app, name=ep.name)
                _REGISTERED_COMMANDS.add(ep.name)
            except Exception as exc:
                logger.debug("Failed to load entry-point plugin %r: %s", ep.name, exc)
    except Exception as e:
        logger.debug("Entry points loading failed: %s", e)

    try:
        from bijux_cli.services.plugins import get_plugins_dir

        plugins_dir = get_plugins_dir()
        for pdir in plugins_dir.iterdir():
            plug_py = pdir / "plugin.py"
            if not plug_py.is_file():
                continue
            mod_name = f"_bijux_cli_plugin_{pdir.name}"
            spec = importlib.util.spec_from_file_location(mod_name, plug_py)
            if not spec or not spec.loader:
                continue
            module = importlib.util.module_from_spec(spec)
            sys.modules[mod_name] = module
            try:
                spec.loader.exec_module(module)
                plugin_app = None
                if hasattr(module, "cli") and callable(module.cli):
                    plugin_app = module.cli()
                elif hasattr(module, "app"):
                    plugin_app = module.app
                else:
                    logger.debug(
                        "Plugin %r has no CLI entrypoint (no cli()/app).", pdir.name
                    )
                    continue
                if isinstance(plugin_app, Typer):
                    app.add_typer(plugin_app, name=pdir.name)
                    _REGISTERED_COMMANDS.add(pdir.name)
                else:
                    logger.debug(
                        "Plugin %r loaded but did not return a Typer app instance.",
                        pdir.name,
                    )
            except Exception as exc:
                logger.debug("Failed to load local plugin %r: %s", pdir.name, exc)
            finally:
                sys.modules.pop(mod_name, None)
    except Exception as e:
        logger.debug("Dynamic plugin discovery failed: %s", e)


def list_registered_command_names() -> list[str]:
    """Returns a list of all registered command names.

    This includes both core commands and any dynamically loaded plugins.

    Returns:
        list[str]: An alphabetically sorted list of all command names.
    """
    return sorted(_REGISTERED_COMMANDS)


__all__ = [
    "register_commands",
    "register_dynamic_plugins",
    "list_registered_command_names",
]
