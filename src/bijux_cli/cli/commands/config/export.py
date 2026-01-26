# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `config export` subcommand for the Bijux CLI.

This module contains the logic for exporting the application's entire current
configuration to a specified destination, which can be a file or standard
output. The output format can be explicitly set to 'env', 'json', or 'yaml',
or it can be inferred from the destination file's extension.

Output Contract:
    * Success (to file):   `{"status": "exported", "file": str, "format": str}`
    * Success (to stdout): The raw exported configuration data is printed directly.
    * Error:               `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1` or `2`: An error occurred during the export process, such as a file
      write error or invalid format request.
"""

from __future__ import annotations

import platform

import typer

from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
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
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ErrorType
from bijux_cli.core.errors import ConfigError
from bijux_cli.core.exit_policy import ExitIntentError
from bijux_cli.core.precedence import current_execution_policy, resolve_exit_intent
from bijux_cli.services.config.contracts import ConfigProtocol


def export_config(
    ctx: typer.Context,
    path: str = typer.Argument(
        ..., help="Destination file – use “-” to write to STDOUT"
    ),
    out_fmt: str = typer.Option(
        None, "--out-format", help="Force output format: env | json | yaml"
    ),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
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
            confirmation payload (file export only).
        fmt (str): The format for the confirmation payload ("json" or "yaml").
        pretty (bool): If True, pretty-prints the confirmation payload.        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    command = "config export"
    effective = current_execution_policy()
    fmt_lower = validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )
    quiet = effective.quiet
    include_runtime = effective.include_runtime
    pretty = effective.pretty

    config_svc = DIContainer.current().resolve(ConfigProtocol)

    try:
        config_svc.export(path, out_fmt)
    except ConfigError as exc:
        code = 2 if getattr(exc, "http_status", 0) == 400 else 1
        intent = resolve_exit_intent(
            message=f"Failed to export config: {exc}",
            code=code,
            failure="export_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.INTERNAL,
            log_level=effective.log_level,
        )
        raise ExitIntentError(intent) from exc

    if path != "-":

        def payload_builder(include_runtime: bool) -> dict[str, object]:
            """Builds the payload confirming a successful export to a file.

            Args:
                include_runtime (bool): If True, includes Python and platform info.

            Returns:
                ConfigExportPayload: The structured payload.
            """
            payload: dict[str, object] = {
                "status": "exported",
                "file": path,
                "format": out_fmt or "auto",
            }
            if include_runtime:
                payload.update(
                    {
                        "python": ascii_safe(
                            platform.python_version(), "python_version"
                        ),
                        "platform": ascii_safe(platform.platform(), "platform"),
                    }
                )
            return payload

        new_run_command(
            command_name=command,
            payload_builder=payload_builder,
            quiet=quiet,
            fmt=fmt_lower,
            pretty=pretty,
            log_level=log_level,
        )
