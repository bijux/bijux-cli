# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config load` subcommand for the Bijux CLI.

This module contains the logic for replacing the application's entire
configuration with the contents of a specified file. It discards any
in-memory settings and loads the new configuration, emitting a structured
confirmation upon success.

Output Contract:
    * Success: `{"status": "loaded", "file": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `2`: The specified file could not be found, read, or parsed.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    parse_global_flags,
)
from bijux_cli.contracts import ConfigProtocol
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer


def load_config(
    ctx: typer.Context,
    path: str = typer.Argument(..., help="Path to load from"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Loads configuration from a specified file.

    This function replaces the current in-memory configuration with the
    contents of the file at the given path. It provides a structured payload
    to confirm the operation was successful.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        path (str): The path to the configuration file to load.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    flags = parse_global_flags()

    quiet = flags["quiet"]
    verbose = flags["verbose"]
    fmt = flags["format"]
    pretty = flags["pretty"]
    debug = flags["debug"]

    include_runtime = verbose
    fmt_lower = fmt.lower()

    command = "config load"

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.load(path)
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to load config: {exc}",
            code=2,
            failure="load_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            extra={"path": path},
        )

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds the payload confirming a successful configuration load.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: The structured payload.
        """
        payload: dict[str, object] = {"status": "loaded", "file": path}
        if include_runtime:
            payload["python"] = ascii_safe(platform.python_version(), "python_version")
            payload["platform"] = ascii_safe(platform.platform(), "platform")
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
