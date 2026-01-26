# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins check` subcommand for the Bijux CLI.

This module contains the logic for performing a health check on a specific
installed plugin. It validates the plugin's files, dynamically imports its
code, and executes a `health()` hook function if available. The result is
reported in a structured, machine-readable format.

Output Contract:
    * Healthy:   `{"plugin": str, "status": "healthy"}`
    * Unhealthy: `{"plugin": str, "status": "unhealthy"}` (exits with code 1)
    * Error:     `{"error": "...", "code": int}` (for pre-check failures)

Exit Codes:
    * `0`: The plugin is healthy.
    * `1`: The plugin is unhealthy, could not be found, or an error occurred
      during import or execution.
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import asyncio
from collections.abc import Mapping
import importlib.util
import inspect
import platform
import sys
import types
from typing import Any

import anyio
import typer

from bijux_cli.cli.core.command import (
    ascii_safe,
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
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.plugins.metadata import get_plugin_metadata


async def check_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Runs a health check on a specific installed plugin.

    This function validates a plugin's structure, dynamically imports its
    `plugin.py` file, and executes its `health()` hook to determine its
    operational status. The final status is emitted as a structured payload.

    Args:
        name (str): The name of the plugin to check.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating the health status or detailing an error.
    """
    command = "plugins check"

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
        meta = await anyio.to_thread.run_sync(get_plugin_metadata, name)
    except Exception as exc:
        raise_exit_intent(
            str(exc),
            code=1,
            failure="metadata_error",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    if not meta.path:
        raise_exit_intent(
            f'Plugin "{name}" has no local health hook',
            code=1,
            failure="health_unavailable",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    plug_dir = meta.path
    plug_py = plug_dir / "plugin.py"
    if not plug_py.is_file():
        raise_exit_intent(
            f'Plugin "{name}" not found',
            code=1,
            failure="not_found",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
            extra={"plugin": name},
        )

    mod_name = f"_bijux_cli_plugin_{name}"
    try:
        spec = importlib.util.spec_from_file_location(mod_name, plug_py)
        if not spec or not spec.loader:
            raise ImportError("Cannot create import spec")
        module = types.ModuleType(mod_name)
        sys.modules[mod_name] = module
        spec.loader.exec_module(module)
    except Exception as exc:
        err = f"Import error: {exc}"
        raise_exit_intent(
            err,
            code=1,
            failure="import_error",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    async def _run_health() -> dict[str, Any]:
        """Isolates and executes the plugin's `health()` hook.

        This function finds and calls the `health()` function within the
        imported plugin module. It handles both synchronous and asynchronous
        hooks, validates their signatures, and safely captures any exceptions
        during execution.

        Returns:
            dict[str, Any]: A dictionary containing the health check result,
                which includes the plugin name and a status ('healthy' or
                'unhealthy'), or an error message.
        """
        hook = getattr(module, "health", None)
        if not callable(hook):
            return {"plugin": name, "error": "No health() hook"}
        try:
            sig = inspect.signature(hook)
            if len(sig.parameters) != 1:
                return {
                    "plugin": name,
                    "error": "health() hook must take exactly one argument (di)",
                }
        except Exception as exc1:
            return {"plugin": name, "error": f"health() signature error: {exc1}"}
        try:
            if asyncio.iscoroutinefunction(hook):
                res = await hook(None)
            else:
                res = await asyncio.to_thread(hook, None)
        except BaseException as exc2:
            return {"plugin": name, "error": str(exc2) or exc2.__class__.__name__}

        if res is True:
            return {"plugin": name, "status": "healthy"}
        if res is False:
            return {"plugin": name, "status": "unhealthy"}
        if isinstance(res, dict) and res.get("status") in ("healthy", "unhealthy"):
            return {"plugin": name, "status": res["status"]}
        return {"plugin": name, "status": "unhealthy"}

    result = await _run_health()
    sys.modules.pop(mod_name, None)
    exit_code = 1 if result.get("status") == "unhealthy" else 0

    if result.get("error"):
        raise_exit_intent(
            result["error"],
            code=1,
            failure="health_error",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    def _build_payload(include: bool) -> Mapping[str, object]:
        """Constructs the final result payload.

        Args:
            include (bool): If True, adds Python and platform info to the payload.

        Returns:
            Mapping[str, object]: The payload containing the health check
                result and optional runtime metadata.
        """
        payload = result
        if include:
            payload["python"] = ascii_safe(platform.python_version(), "python_version")
            payload["platform"] = ascii_safe(platform.platform(), "platform")
        return payload

    new_run_command(
        command_name=command,
        payload_builder=_build_payload,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level_value,
        exit_code=exit_code,
    )
