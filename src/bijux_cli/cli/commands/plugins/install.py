# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins install` subcommand for the Bijux CLI.

This module installs a plugin from PyPI by package name only. It validates that
the installed package exposes a `bijux_cli.plugins` entry point and that its
metadata declares compatibility with the running bijux-cli version.

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

import os
from pathlib import Path
import subprocess  # noqa: S603
import sys

import typer

from bijux_cli.cli.commands.plugins.validation import PLUGIN_NAME_RE
from bijux_cli.cli.commands.utilities import (
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
from bijux_cli.plugins.metadata import (
    discover_plugins,
    invalidate_plugin_cache,
    plugins_for_package,
)


def install_plugin(
    name: str = typer.Argument(..., help="PyPI package name"),
    dry_run: bool = typer.Option(False, "--dry-run"),
    force: bool = typer.Option(False, "--force", "-F"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(False, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Installs a plugin from PyPI by package name.

    Args:
        name (str): The package name to install from PyPI.
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
    command = "plugins install"

    fmt_lower = validate_common_flags(fmt, command, quiet)
    if Path(name).exists():
        emit_error_and_exit(
            "Local paths are not supported; use a PyPI package name.",
            code=1,
            failure="local_path_not_supported",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    if not PLUGIN_NAME_RE.fullmatch(name) or not name.isascii():
        emit_error_and_exit(
            "Invalid package name: only ASCII letters, digits, dash and underscore are allowed.",
            code=1,
            failure="invalid_name",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    if dry_run:
        payload = {"status": "dry-run", "package": name}
    else:
        invalidate_plugin_cache()
        cmd = [sys.executable, "-m", "pip", "install", name]
        if force:
            cmd.append("--upgrade")
        env = os.environ.copy()
        env.setdefault("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        proc = subprocess.run(cmd, env=env, capture_output=True, text=True)
        if proc.returncode != 0:
            detail = proc.stderr.strip() or proc.stdout.strip()
            emit_error_and_exit(
                f"pip install failed: {detail}",
                code=1,
                failure="pip_install_failed",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

        invalidate_plugin_cache()
        try:
            discover_plugins()
            plugins = plugins_for_package(name)
        except Exception as exc:
            emit_error_and_exit(
                str(exc),
                code=1,
                failure="metadata_error",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

        if not plugins:
            emit_error_and_exit(
                "Package installed but no bijux_cli.plugins entry point found.",
                code=1,
                failure="entrypoint_missing",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

        payload = {
            "status": "installed",
            "package": name,
            "plugins": [p.name for p in plugins],
        }

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
