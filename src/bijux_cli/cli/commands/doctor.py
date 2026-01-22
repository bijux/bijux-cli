# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `doctor` command for the Bijux CLI.

This module provides the functionality for the `bijux doctor` command, which runs
a series of health diagnostics on the CLI's operating environment. It checks for
common configuration issues and reports a summary of its findings in a
structured, machine-readable format suitable for automation.

Output Contract:
    * Success: `{"status": str, "summary": list[str]}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success (command ran without errors, regardless of health status).
    * `1`: Internal or fatal error (e.g., dependency injection failure).
    * `2`: CLI argument or flag error.
"""

from __future__ import annotations

from collections.abc import Mapping
import os
import platform

import typer

from bijux_cli.app.async_exec import AsyncTyper
from bijux_cli.app.di import DIContainer
from bijux_cli.cli.constants import (
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.emit import emit_error_and_exit
from bijux_cli.cli.output import get_execution_policy, new_run_command
from bijux_cli.cli.validation import (
    ascii_safe,
    normalize_format,
    validate_common_flags,
)
from bijux_cli.core.contracts import Emitter
from bijux_cli.services.contracts import TelemetryProtocol

typer.core.rich = None  # type: ignore[attr-defined,assignment]

doctor_app = AsyncTyper(
    name="doctor",
    help="Run CLI health diagnostics and environment checks.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool) -> Mapping[str, object]:
    """Builds the payload summarizing CLI environment health.

    This function performs a series of checks on the environment and aggregates
    the findings into a structured payload.

    Args:
        include_runtime (bool): If True, appends Python and platform version
            information to the payload.

    Returns:
        Mapping[str, object]: A dictionary containing the health status, a
            summary of findings, and optional runtime details.
    """
    healthy = True
    summary: list[str] = []

    if not os.environ.get("PATH", ""):
        healthy = False
        summary.append("Environment PATH is empty")

    if os.environ.get("BIJUXCLI_TEST_FORCE_UNHEALTHY") == "1":
        healthy = False
        summary.append("Forced unhealthy by test environment")

    if not summary:
        summary.append(
            "All core checks passed" if healthy else "Unknown issue detected"
        )

    payload: dict[str, object] = {
        "status": "healthy" if healthy else "unhealthy",
        "summary": summary,
    }

    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")

    return payload


@doctor_app.callback(invoke_without_command=True)
def doctor(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux doctor` command.

    This function orchestrates the health check process. It validates all CLI
    flags, performs critical pre-flight checks (like dependency availability),
    and then invokes the main run utility to build and emit the health payload.

    Args:
        ctx (typer.Context): The Typer context for managing command state.
        quiet (bool): If True, suppresses all output; the exit code is the
            primary indicator of the outcome.
        verbose (bool): If True, includes Python and platform details in the
            output payload.
        fmt (str): The output format, either "json" or "yaml". Defaults to "json".
        pretty (bool): If True, pretty-prints the output for human readability.
        debug (bool): If True, enables debug diagnostics, implying `verbose`
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Exits the application with a contract-compliant status code
            and payload upon any error, such as invalid arguments or an
            internal system failure.
    """
    if ctx.invoked_subcommand:
        return

    command = "doctor"
    _ = (quiet, verbose, log_level, pretty, fmt)
    policy = get_execution_policy()
    quiet = policy.quiet
    verbose = policy.verbose
    debug = policy.log_level == "debug"
    pretty = policy.pretty
    include_runtime = policy.include_runtime
    fmt_lower = normalize_format(fmt) or "json"

    validate_common_flags(fmt, command, quiet)

    if ctx.args:
        stray = ctx.args[0]
        msg = (
            f"No such option: {stray}"
            if stray.startswith("-")
            else f"Too many arguments: {' '.join(ctx.args)}"
        )
        emit_error_and_exit(
            msg,
            code=2,
            failure="args",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    try:
        DIContainer.current().resolve(Emitter)
        DIContainer.current().resolve(TelemetryProtocol)
    except Exception as exc:
        emit_error_and_exit(
            str(exc),
            code=1,
            failure="internal",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    new_run_command(
        command_name=command,
        payload_builder=_build_payload,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
