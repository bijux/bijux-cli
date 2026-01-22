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
import subprocess  # noqa: S603
import sys
import unicodedata

import typer

from bijux_cli.cli.commands.plugins.validation import refuse_on_symlink
from bijux_cli.cli.commands.utilities import (
    emit_error_and_exit,
    new_run_command,
    resolve_command_config,
)
from bijux_cli.cli.commands.utilities import (
    validate_common_flags as validate_common_flags,
)
from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.plugins import get_plugins_dir
from bijux_cli.plugins.metadata import (
    get_plugin_metadata,
    invalidate_plugin_cache,
)


def uninstall_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
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

    validate_common_flags(fmt, command, quiet)
    effective, _, fmt_lower = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    debug = effective.log_level == "debug"
    pretty = effective.pretty
    try:
        meta = get_plugin_metadata(name)
    except Exception:
        meta = None

    if meta and meta.source == "entrypoint" and meta.dist_name:
        cmd = [sys.executable, "-m", "pip", "uninstall", "-y", meta.dist_name]
        proc = subprocess.run(cmd, capture_output=True, text=True)  # noqa: S603
        if proc.returncode != 0:
            detail = proc.stderr.strip() or proc.stdout.strip()
            emit_error_and_exit(
                f"pip uninstall failed: {detail}",
                code=1,
                failure="pip_uninstall_failed",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective.include_runtime,
                debug=debug,
            )
        invalidate_plugin_cache()
        payload = {"status": "uninstalled", "plugin": name}
        new_run_command(
            command_name=command,
            payload_builder=lambda include: payload,
            quiet=quiet,
            verbose=verbose,
            fmt=fmt_lower,
            pretty=pretty,
            log_level=log_level,
        )
        return

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
            include_runtime=effective.include_runtime,
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
            include_runtime=effective.include_runtime,
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
                include_runtime=effective.include_runtime,
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
                include_runtime=effective.include_runtime,
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
                    include_runtime=effective.include_runtime,
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
                    include_runtime=effective.include_runtime,
                    debug=debug,
                )

    invalidate_plugin_cache()
    payload = {"status": "uninstalled", "plugin": name}

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
