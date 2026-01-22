# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config unset` subcommand for the Bijux CLI.

This module contains the logic for removing a key-value pair from the active
configuration store. It provides a structured, machine-readable response to
confirm the deletion or report an error, such as if the key does not exist.

Output Contract:
    * Success: `{"status": "deleted", "key": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred while accessing the configuration.
    * `2`: The specified key was not found in the configuration.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.app.di import DIContainer
from bijux_cli.cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    parse_global_flags,
    resolve_command_config,
)
from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.services.config.contracts import ConfigProtocol


def unset_config(
    ctx: typer.Context,
    key: str = typer.Argument(..., help="Key to remove"),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Removes a key from the active configuration store.

    This function orchestrates the `unset` operation. It manually parses global
    flags, resolves the configuration service, attempts to remove the specified
    key, and then uses the `new_run_command` helper to emit a structured
    payload confirming the action.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        key (str): The configuration key to remove.
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
    command = "config unset"
    effective, _, fmt_lower = resolve_command_config(
        command=command,
        quiet=flags["quiet"],
        verbose=flags["verbose"],
        log_level=flags["log_level"],
        fmt=flags["format"],
        pretty=flags["pretty"],
    )
    quiet = effective.quiet
    verbose = effective.verbose_level > 0
    debug = effective.log_level == "debug"
    pretty = effective.pretty
    include_runtime = effective.include_runtime

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.unset(key)
    except KeyError:
        emit_error_and_exit(
            f"Config key not found: {key}",
            code=2,
            failure="not_found",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            extra={"key": key},
        )
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to unset config: {exc}",
            code=1,
            failure="unset_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds the payload confirming a key was deleted.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: The structured payload.
        """
        payload: dict[str, object] = {"status": "deleted", "key": key}
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
        log_level=log_level,
    )
