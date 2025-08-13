# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins install` subcommand for the Bijux CLI.

This module contains the logic for installing a new plugin by copying its
source directory into the CLI's plugins folder. The process is designed to be
atomic and safe, incorporating validation of the plugin's name and metadata,
version compatibility checks against the current CLI version, and file locking
to prevent race conditions during installation.

Output Contract:
    * Install Success: `{"status": "installed", "plugin": str, "dest": str}`
    * Dry Run Success: `{"status": "dry-run", "plugin": str, ...}`
    * Error:           `{"error": "...", "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal error occurred (e.g., source not found, invalid name,
      version incompatibility, filesystem error).
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

from collections.abc import Iterator
import contextlib
import errno
import fcntl
from pathlib import Path
import shutil
import tempfile

import typer

from bijux_cli.commands.plugins.utils import (
    PLUGIN_NAME_RE,
    ignore_hidden_and_broken_symlinks,
    parse_required_cli_version,
    refuse_on_symlink,
)
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


def install_plugin(
    path: str = typer.Argument(..., help="Path to plugin directory"),
    dry_run: bool = typer.Option(False, "--dry-run"),
    force: bool = typer.Option(False, "--force", "-F"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(False, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Installs a plugin from a local source directory.

    This function orchestrates the plugin installation process. It validates
    the source path and plugin name, checks for version compatibility, handles
    pre-existing plugins via the `--force` flag, and performs an atomic copy
    into the plugins directory using a file lock and temporary directory.

    Args:
        path (str): The source path to the plugin directory to install.
        dry_run (bool): If True, simulates the installation without making changes.
        force (bool): If True, overwrites an existing plugin of the same name.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes runtime metadata in error payloads.
        fmt (str): The output format for confirmation or error messages.
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    from packaging.specifiers import SpecifierSet

    from bijux_cli.__version__ import version as cli_version

    command = "plugins install"

    fmt_lower = validate_common_flags(fmt, command, quiet)
    plugins_dir = get_plugins_dir()
    refuse_on_symlink(plugins_dir, command, fmt_lower, quiet, verbose, debug)

    src = Path(path).expanduser()
    try:
        src = src.resolve()
    except (FileNotFoundError, OSError, RuntimeError):
        src = src.absolute()
    if not src.exists() or not src.is_dir():
        emit_error_and_exit(
            "Source not found",
            code=1,
            failure="source_not_found",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    plugin_name = src.name

    if not PLUGIN_NAME_RE.fullmatch(plugin_name) or not plugin_name.isascii():
        emit_error_and_exit(
            "Invalid plugin name: only ASCII letters, digits, dash and underscore are allowed.",
            code=1,
            failure="invalid_name",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    dest = plugins_dir / plugin_name

    try:
        plugins_dir.mkdir(parents=True, exist_ok=True)
    except Exception as exc:
        emit_error_and_exit(
            f"Cannot create plugins dir '{plugins_dir}': {exc}",
            code=1,
            failure="create_dir_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    lock_file = plugins_dir / ".bijux_install.lock"

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
        with fp.open("w") as fh:
            fcntl.flock(fh, fcntl.LOCK_EX)
            try:
                yield
            finally:
                fcntl.flock(fh, fcntl.LOCK_UN)

    with _lock(lock_file):
        if plugins_dir.is_symlink():
            emit_error_and_exit(
                f"Refusing to install: plugins dir '{plugins_dir}' is a symlink.",
                code=1,
                failure="symlink_dir",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

        if dest.exists():
            if not force:
                emit_error_and_exit(
                    f"Plugin '{plugin_name}' already installed. Use --force.",
                    code=1,
                    failure="already_installed",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=verbose,
                    debug=debug,
                )
            try:
                if dest.is_dir():
                    shutil.rmtree(dest)
                else:
                    dest.unlink()
            except Exception as exc:
                emit_error_and_exit(
                    f"Unable to remove existing '{dest}': {exc}",
                    code=1,
                    failure="remove_failed",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=verbose,
                    debug=debug,
                )

        plugin_py = src / "plugin.py"
        if not plugin_py.exists():
            emit_error_and_exit(
                "plugin.py not found in plugin directory",
                code=1,
                failure="plugin_py_missing",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )
        version_spec = parse_required_cli_version(plugin_py)
        if version_spec:
            try:
                spec = SpecifierSet(version_spec)
                if not spec.contains(cli_version, prereleases=True):
                    emit_error_and_exit(
                        f"Incompatible CLI version: plugin requires '{version_spec}', but you have '{cli_version}'",
                        code=1,
                        failure="incompatible_version",
                        command=command,
                        fmt=fmt_lower,
                        quiet=quiet,
                        include_runtime=verbose,
                        debug=debug,
                    )
            except Exception as exc:
                emit_error_and_exit(
                    f"Invalid version specifier in plugin: '{version_spec}'. {exc}",
                    code=1,
                    failure="invalid_specifier",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=verbose,
                    debug=debug,
                )

        if dry_run:
            payload = {
                "status": "dry-run",
                "plugin": plugin_name,
                "source": str(src),
                "dest": str(dest),
            }
        else:
            with tempfile.TemporaryDirectory(dir=plugins_dir) as td:
                tmp_dst = Path(td) / plugin_name
                try:
                    shutil.copytree(
                        src,
                        tmp_dst,
                        symlinks=True,
                        ignore=ignore_hidden_and_broken_symlinks,
                    )
                except OSError as exc:
                    if exc.errno == errno.ENOSPC or "No space left on device" in str(
                        exc
                    ):
                        emit_error_and_exit(
                            "Disk full during plugin install",
                            code=1,
                            failure="disk_full",
                            command=command,
                            fmt=fmt_lower,
                            quiet=quiet,
                            include_runtime=verbose,
                            debug=debug,
                        )
                    if exc.errno == errno.EACCES or "Permission denied" in str(exc):
                        emit_error_and_exit(
                            "Permission denied during plugin install",
                            code=1,
                            failure="permission_denied",
                            command=command,
                            fmt=fmt_lower,
                            quiet=quiet,
                            include_runtime=verbose,
                            debug=debug,
                        )
                    emit_error_and_exit(
                        f"OSError during plugin install: {exc!r}",
                        code=1,
                        failure="os_error",
                        command=command,
                        fmt=fmt_lower,
                        quiet=quiet,
                        include_runtime=verbose,
                        debug=debug,
                    )
                if not (tmp_dst / "plugin.py").is_file():
                    emit_error_and_exit(
                        f"plugin.py not found in '{tmp_dst}'",
                        code=1,
                        failure="plugin_py_missing_after_copy",
                        command=command,
                        fmt=fmt_lower,
                        quiet=quiet,
                        include_runtime=verbose,
                        debug=debug,
                    )
                shutil.move(str(tmp_dst), dest)
            payload = {"status": "installed", "plugin": plugin_name, "dest": str(dest)}

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
