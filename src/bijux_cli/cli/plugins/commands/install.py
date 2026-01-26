# SPDX-License-Identifier: Apache-2.0
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
import shutil
import subprocess  # noqa: S603  # nosec B404 - intentional CLI invocation
import sys

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
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.plugins import get_plugins_dir
from bijux_cli.plugins import install_plugin as install_local_plugin
from bijux_cli.plugins.catalog import PLUGIN_NAME_RE
from bijux_cli.plugins.metadata import (
    discover_plugins,
    get_plugin_metadata,
    invalidate_plugin_cache,
    plugins_for_package,
)


def install_plugin(
    name: str = typer.Argument(..., help="PyPI package name"),
    dry_run: bool = typer.Option(False, "--dry-run"),
    force: bool = typer.Option(False, "--force", "-F"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(False, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Installs a plugin from PyPI by package name.

    Args:
        name (str): The package name to install from PyPI.
        dry_run (bool): If True, simulates the installation without making changes.
        force (bool): If True, overwrites an existing plugin of the same name.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format for confirmation or error messages.
        pretty (bool): If True, pretty-prints the output.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins install"

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
    local_path = Path(name)
    if not local_path.exists() and (
        not PLUGIN_NAME_RE.fullmatch(name) or not name.isascii()
    ):
        raise_exit_intent(
            "Invalid package name: only ASCII letters, digits, dash and underscore are allowed.",
            code=1,
            failure="invalid_name",
            error_type=ErrorType.USER_INPUT,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    if dry_run:
        payload: dict[str, object] = {"status": "dry-run", "package": name}
    elif local_path.exists():
        invalidate_plugin_cache()
        try:
            install_local_plugin(str(local_path), force=force)
            invalidate_plugin_cache()
            discover_plugins()
            meta = get_plugin_metadata(local_path.name)
        except Exception as exc:
            plug_dir = get_plugins_dir() / local_path.name
            if plug_dir.exists():
                shutil.rmtree(plug_dir, ignore_errors=True)
            raise_exit_intent(
                str(exc),
                code=1,
                failure="metadata_error",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
            )
        payload = {
            "status": "installed",
            "package": str(local_path),
            "plugins": [meta.name],
        }
    else:
        invalidate_plugin_cache()
        cmd = [sys.executable, "-m", "pip", "install", name]
        if force:
            cmd.append("--upgrade")
        env = os.environ.copy()
        env.setdefault("PIP_DISABLE_PIP_VERSION_CHECK", "1")
        proc = subprocess.run(  # noqa: S603  # nosec B603 - controlled command list
            cmd,
            env=env,
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            detail = proc.stderr.strip() or proc.stdout.strip()
            raise_exit_intent(
                f"pip install failed: {detail}",
                code=1,
                failure="pip_install_failed",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
            )

        invalidate_plugin_cache()
        try:
            discover_plugins()
            plugins = plugins_for_package(name)
        except Exception as exc:
            raise_exit_intent(
                str(exc),
                code=1,
                failure="metadata_error",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
            )

        if not plugins:
            raise_exit_intent(
                "Package installed but no bijux_cli.plugins entry point found.",
                code=1,
                failure="entrypoint_missing",
                error_type=ErrorType.PLUGIN,
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level_value,
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
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
