# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config list` subcommand for the Bijux CLI.

This module contains the logic for listing all keys currently defined in the
active configuration store. It retrieves the keys and presents them in a
structured, machine-readable list format.

Output Contract:
    * Success: `{"items": [{"key": str}, ...]}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred while accessing the configuration.
"""

from __future__ import annotations

from collections.abc import Mapping
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


def list_config(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Lists all configuration keys from the active configuration store.

    This function retrieves all defined keys, sorts them, and then uses the
    `new_run_command` helper to emit them in a structured payload.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
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

    command = "config list"

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        keys = config_svc.list_keys()
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to list config: {exc}",
            code=1,
            failure="list_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    def payload_builder(include_runtime: bool) -> Mapping[str, object]:
        """Builds a payload containing the list of configuration keys.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            Mapping[str, object]: A dictionary containing a sorted list of
                keys under an "items" field, plus optional runtime metadata.
        """
        payload: dict[str, object] = {
            "items": [{"key": k} for k in sorted(keys, key=str.lower)]
        }
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
