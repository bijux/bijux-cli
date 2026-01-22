# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config export` subcommand for the Bijux CLI.

This module contains the logic for exporting the application's entire current
configuration to a specified destination, which can be a file or standard
output. The output format can be explicitly set to 'env', 'json', or 'yaml',
or it can be inferred from the destination file's extension.

Output Contract:
    * Success (to file):   `{"status": "exported", "file": str, "format": str}`
    * Success (to stdout): The raw exported configuration data is printed directly.
    * Verbose (to file):   Adds `{"python": str, "platform": str}` to the payload.
    * Error:               `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1` or `2`: An error occurred during the export process, such as a file
      write error or invalid format request.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.app.di import DIContainer
from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.emit import emit_error_and_exit
from bijux_cli.cli.output import new_run_command, resolve_command_config
from bijux_cli.cli.validation import ascii_safe
from bijux_cli.core.errors import CommandError
from bijux_cli.services.config.contracts import ConfigProtocol


def export_config(
    ctx: typer.Context,
    path: str = typer.Argument(
        ..., help="Destination file – use “-” to write to STDOUT"
    ),
    out_fmt: str = typer.Option(
        None, "--out-format", help="Force output format: env | json | yaml"
    ),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Exports the current configuration to a file or standard output.

    This function writes all configuration key-value pairs to a specified
    destination. If the destination is a file path, a structured JSON/YAML
    confirmation message is printed to stdout upon success. If the destination
    is "-", the raw exported configuration is printed directly to stdout.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        path (str): The destination file path, or "-" for standard output.
        out_fmt (str): The desired output format ('env', 'json', 'yaml'). If
            unspecified, it is inferred from the file extension.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the
            confirmation payload (file export only).
        fmt (str): The format for the confirmation payload ("json" or "yaml").
        pretty (bool): If True, pretty-prints the confirmation payload.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    command = "config export"
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
    include_runtime = effective.include_runtime

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.export(path, out_fmt)
    except CommandError as exc:
        code = 2 if getattr(exc, "http_status", 0) == 400 else 1
        emit_error_and_exit(
            f"Failed to export config: {exc}",
            code=code,
            failure="export_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    if path != "-":

        def payload_builder(include_runtime: bool) -> dict[str, object]:
            """Builds the payload confirming a successful export to a file.

            Args:
                include_runtime (bool): If True, includes Python and platform info.

            Returns:
                dict[str, object]: The structured payload.
            """
            payload: dict[str, object] = {
                "status": "exported",
                "file": path,
                "format": out_fmt or "auto",
            }
            if include_runtime:
                payload["python"] = ascii_safe(
                    platform.python_version(), "python_version"
                )
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


def parse_global_flags() -> dict[str, object]:
    """Legacy shim for tests; do not use in command logic."""
    from bijux_cli.cli.flags import parse_and_apply_global_flags

    flags, _ = parse_and_apply_global_flags([])
    return flags
