# SPDX-License-Identifier: Apache-2.0
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
import subprocess  # noqa: S603  # nosec B404 - intentional CLI invocation
import sys
import unicodedata

import typer

from bijux_cli.cli.core.command import (
    new_run_command,
    raise_exit_intent,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
)
from bijux_cli.cli.core.help_text import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
)
from bijux_cli.cli.plugins.commands.validation import refuse_on_symlink
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.plugins import get_plugins_dir
from bijux_cli.plugins.metadata import get_plugin_metadata, invalidate_plugin_cache


def uninstall_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Removes an installed plugin by deleting its directory.

    This function locates the plugin directory by name, performs several safety
    checks, acquires a file lock to ensure atomicity, and then permanently
    removes the plugin from the filesystem.

    Args:
        name (str): The name of the plugin to uninstall. The match is
            case-sensitive and Unicode-aware.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format for confirmation or error messages.
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins uninstall"

    policy = current_execution_policy()
    quiet = policy.quiet
    include_runtime = policy.include_runtime
    log_level_value = policy.log_level
    pretty = policy.pretty
    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )
    try:
        meta = get_plugin_metadata(name)
    except Exception:
        meta = None

    if meta and meta.source == "entrypoint" and meta.dist_name:
        cmd = [sys.executable, "-m", "pip", "uninstall", "-y", meta.dist_name]
        proc = subprocess.run(  # noqa: S603  # nosec B603 - controlled command list
            cmd,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            detail = proc.stderr.strip() or proc.stdout.strip()
            raise_exit_intent(
                f"pip uninstall failed: {detail}",
                code=1,
                failure="pip_uninstall_failed",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
            )
        invalidate_plugin_cache()
        payload = {"status": "uninstalled", "plugin": name}
        new_run_command(
            command_name=command,
            payload_builder=lambda include: payload,
            quiet=quiet,
            fmt=fmt_lower,
            pretty=pretty,
            log_level=log_level_value,
        )

    plugins_dir = get_plugins_dir()
    refuse_on_symlink(plugins_dir, command, fmt_lower, quiet, log_level_value)

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
        raise_exit_intent(
            f"Could not list plugins dir '{plugins_dir}': {exc}",
            code=1,
            failure="list_failed",
            error_type=ErrorType.PLUGIN,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    if not plugin_dirs:
        raise_exit_intent(
            f"Plugin '{name}' is not installed.",
            code=1,
            failure="not_installed",
            error_type=ErrorType.PLUGIN,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
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
            raise_exit_intent(
                f"Plugin path '{plug_path}' is a symlink. Refusing to uninstall.",
                code=1,
                failure="symlink_path",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
            )
        elif not plug_path.is_dir():
            raise_exit_intent(
                f"Plugin path '{plug_path}' is not a directory.",
                code=1,
                failure="not_dir",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
            )
        else:
            try:
                shutil.rmtree(plug_path)
            except PermissionError:
                raise_exit_intent(
                    f"Permission denied removing '{plug_path}'",
                    code=1,
                    failure="permission_denied",
                    error_type=ErrorType.PLUGIN,
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=include_runtime,
                    log_level=log_level_value,
                )
            except Exception as exc:
                raise_exit_intent(
                    f"Failed to remove '{plug_path}': {exc}",
                    code=1,
                    failure="remove_failed",
                    error_type=ErrorType.PLUGIN,
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=include_runtime,
                    log_level=log_level_value,
                )

    invalidate_plugin_cache()
    payload = {"status": "uninstalled", "plugin": name}

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
