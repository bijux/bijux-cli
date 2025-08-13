# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins uninstall` subcommand for the Bijux CLI.

This module contains the logic for permanently removing an installed plugin
from the filesystem. The operation locates the plugin directory by its exact
name, performs security checks (e.g., refusing to act on symbolic links),
and uses a file lock to ensure atomicity before deleting the directory.

Output Contract:
    * Success: `{"status": "uninstalled", "plugin": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal error occurred (e.g., plugin not found, permission denied,
      filesystem error).
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

from collections.abc import Iterator
import contextlib
import fcntl
from pathlib import Path
import shutil
import unicodedata

import typer

from bijux_cli.commands.plugins.utils import refuse_on_symlink
from bijux_cli.commands.utilities import (
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.services.plugins import get_plugins_dir


def uninstall_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Removes an installed plugin by deleting its directory.

    This function locates the plugin directory by name, performs several safety
    checks, acquires a file lock to ensure atomicity, and then permanently
    removes the plugin from the filesystem.

    Args:
        name (str): The name of the plugin to uninstall. The match is
            case-sensitive and Unicode-aware.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in error outputs.
        fmt (str): The output format for confirmation or error messages.
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins uninstall"

    fmt_lower = validate_common_flags(fmt, command, quiet)
    plugins_dir = get_plugins_dir()
    refuse_on_symlink(plugins_dir, command, fmt_lower, quiet, verbose, debug)

    lock_file = plugins_dir / ".bijux_install.lock"

    plugin_dirs: list[Path] = []
    try:
        plugin_dirs = [
            p
            for p in plugins_dir.iterdir()
            if p.is_dir()
            and unicodedata.normalize("NFC", p.name)
            == unicodedata.normalize("NFC", name)
        ]
    except Exception as exc:
        emit_error_and_exit(
            f"Could not list plugins dir '{plugins_dir}': {exc}",
            code=1,
            failure="list_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    if not plugin_dirs:
        emit_error_and_exit(
            f"Plugin '{name}' is not installed.",
            code=1,
            failure="not_installed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    plug_path = plugin_dirs[0]

    @contextlib.contextmanager
    def _lock(fp: Path) -> Iterator[None]:
        """Provides an exclusive, non-blocking file lock.

        This context manager attempts to acquire a lock on the specified file.
        It is used to ensure atomic filesystem operations within the plugins
        directory.

        Args:
            fp (Path): The path to the file to lock.

        Yields:
            None: Yields control to the `with` block once the lock is acquired.
        """
        fp.parent.mkdir(parents=True, exist_ok=True)
        with fp.open("w") as fh:
            fcntl.flock(fh, fcntl.LOCK_EX)
            try:
                yield
            finally:
                fcntl.flock(fh, fcntl.LOCK_UN)

    with _lock(lock_file):
        if not plug_path.exists():
            pass
        elif plug_path.is_symlink():
            emit_error_and_exit(
                f"Plugin path '{plug_path}' is a symlink. Refusing to uninstall.",
                code=1,
                failure="symlink_path",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )
        elif not plug_path.is_dir():
            emit_error_and_exit(
                f"Plugin path '{plug_path}' is not a directory.",
                code=1,
                failure="not_dir",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )
        else:
            try:
                shutil.rmtree(plug_path)
            except PermissionError:
                emit_error_and_exit(
                    f"Permission denied removing '{plug_path}'",
                    code=1,
                    failure="permission_denied",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=verbose,
                    debug=debug,
                )
            except Exception as exc:
                emit_error_and_exit(
                    f"Failed to remove '{plug_path}': {exc}",
                    code=1,
                    failure="remove_failed",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=verbose,
                    debug=debug,
                )

    payload = {"status": "uninstalled", "plugin": name}

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
