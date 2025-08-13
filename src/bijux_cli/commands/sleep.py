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

from collections.abc import Mapping
import platform
import time

import typer

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.contracts import ConfigProtocol
from bijux_cli.core.constants import (
    DEFAULT_COMMAND_TIMEOUT,
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer

typer.core.rich = None  # type: ignore[attr-defined,assignment]

sleep_app = typer.Typer(  # pytype: skip-file
    name="sleep",
    help="Pause execution for a specified duration.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool, slept: float) -> Mapping[str, object]:
    """Constructs the structured payload for the sleep command.

    Args:
        include_runtime (bool): If True, includes Python version and platform
            information in the payload.
        slept (float): The number of seconds the command slept.

    Returns:
        Mapping[str, object]: A dictionary containing the sleep duration and
            optional runtime details.
    """
    payload: dict[str, object] = {"slept": slept}
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
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
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
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

    fmt_lower = validate_common_flags(fmt, command, quiet)

    if seconds < 0:
        emit_error_and_exit(
            "sleep length must be non-negative",
            code=2,
            failure="negative",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
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
            include_runtime=verbose,
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
            include_runtime=verbose,
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
        debug=debug,
    )
