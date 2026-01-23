# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `sleep` command for the Bijux CLI.

This module provides a simple command to pause execution for a specified duration.
It is primarily used for scripting, testing, or rate-limiting operations within
automated workflows. The command returns a structured payload confirming the
duration slept.

Output Contract:
    * Success: `{"slept": float}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: Internal or configuration-related error.
    * `2`: Invalid argument (e.g., negative duration) or timeout exceeded.
"""

from __future__ import annotations

import platform
import time

import typer

from bijux_cli.cli.constants import (
    DEFAULT_COMMAND_TIMEOUT,
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.cli.emit import emit_error_and_exit
from bijux_cli.cli.output import new_run_command, resolve_command_config
from bijux_cli.cli.payloads import SleepPayload
from bijux_cli.cli.validation import ascii_safe
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import LogLevel
from bijux_cli.core.runtime import AsyncTyper
from bijux_cli.services.config.contracts import ConfigProtocol

typer.core.rich = None  # type: ignore[attr-defined,assignment]

sleep_app = AsyncTyper(
    name="sleep",
    help="Pause execution for a specified duration.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool, slept: float) -> SleepPayload:
    """Constructs the structured payload for the sleep command.

    Args:
        include_runtime (bool): If True, includes Python version and platform
            information in the payload.
        slept (float): The number of seconds the command slept.

    Returns:
        Mapping[str, object]: A dictionary containing the sleep duration and
            optional runtime details.
    """
    payload = SleepPayload(slept=slept)
    if include_runtime:
        return SleepPayload(
            slept=slept,
            python=ascii_safe(platform.python_version(), "python_version"),
            platform=ascii_safe(platform.platform(), "platform"),
        )
    return payload


@sleep_app.callback(invoke_without_command=True)
def sleep(
    ctx: typer.Context,
    seconds: float = typer.Option(
        ..., "--seconds", "-s", help="Duration in seconds (must be ≥ 0)"
    ),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux sleep` command.

    This function validates the requested sleep duration against configuration
    limits, pauses execution, and then emits a structured payload confirming
    the duration.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        seconds (float): The duration in seconds to pause execution. Must be
            non-negative and not exceed the configured command timeout.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python and platform details in the
            output payload.
        fmt (str): The output format, either "json" or "yaml". Defaults to "json".
        pretty (bool): If True, pretty-prints the output for human readability.
        debug (bool): If True, enables debug diagnostics, implying `verbose`
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Exits with a contract-compliant status code and payload
            upon any error, such as a negative sleep duration or a timeout
            violation.
    """
    command = "sleep"

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
    debug = effective.log_level == LogLevel.DEBUG
    pretty = effective.pretty

    if seconds < 0:
        emit_error_and_exit(
            "sleep length must be non-negative",
            code=2,
            failure="negative",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            debug=debug,
        )

    cfg: ConfigProtocol = DIContainer.current().resolve(ConfigProtocol)

    try:
        timeout = float(cfg.get("BIJUXCLI_COMMAND_TIMEOUT", DEFAULT_COMMAND_TIMEOUT))
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to read timeout: {exc}",
            code=1,
            failure="config",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            debug=debug,
        )

    if seconds > timeout:
        emit_error_and_exit(
            "Command timed out because sleep duration exceeded the configured timeout.",
            code=2,
            failure="timeout",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective.include_runtime,
            debug=debug,
        )

    time.sleep(seconds)

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include, seconds),
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level,
    )
