# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `version` command for the Bijux CLI.

This module reports the CLI's version and runtime environment information.
The output is machine-readable, available in JSON or YAML, and is designed
to be safe for automation and scripting by adhering to a strict output
contract and ASCII hygiene.

Output Contract:
    * Success: `{"version": str}`
    * Verbose: Adds `{"python": str, "platform": str, "timestamp": float}`.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: Internal or fatal error.
    * `2`: CLI argument, flag, or format error.
    * `3`: ASCII or encoding error.
"""

from __future__ import annotations

import os
import platform
import re
import time

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
from bijux_cli.cli.output import new_run_command, resolve_command_config
from bijux_cli.cli.payloads import VersionPayload
from bijux_cli.cli.validation import ascii_safe, validate_common_flags
from bijux_cli.core.contracts import Emitter
from bijux_cli.services.contracts import TelemetryProtocol
from bijux_cli.version import __version__ as cli_version

typer.core.rich = None  # type: ignore[attr-defined,assignment]

version_app = AsyncTyper(
    name="version",
    help="Show the CLI version.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool) -> VersionPayload:
    """Builds the structured payload for the version command.

    The version can be overridden by the `BIJUXCLI_VERSION` environment
    variable, which is validated for correctness.

    Args:
        include_runtime (bool): If True, appends Python/platform details
            and a timestamp to the payload.

    Returns:
        Mapping[str, object]: A dictionary containing the CLI version and
            optional runtime metadata.

    Raises:
        ValueError: If `BIJUXCLI_VERSION` is set but is empty, too long,
            contains non-ASCII characters, or is not a valid semantic version.
    """
    version_env = os.environ.get("BIJUXCLI_VERSION")
    if version_env is not None:
        if not (1 <= len(version_env) <= 1024):
            raise ValueError("BIJUXCLI_VERSION is empty or too long")
        if not all(ord(c) < 128 for c in version_env):
            raise ValueError("BIJUXCLI_VERSION contains non-ASCII")
        if not re.fullmatch(r"\d+\.\d+\.\d+", version_env):
            raise ValueError("BIJUXCLI_VERSION is not valid semantic version (x.y.z)")
        version_ = version_env
    else:
        version_ = cli_version

    payload = VersionPayload(version=ascii_safe(version_, "BIJUXCLI_VERSION"))

    if include_runtime:
        return VersionPayload(
            version=payload.version,
            python=ascii_safe(platform.python_version(), "python_version"),
            platform=ascii_safe(platform.platform(), "platform"),
            timestamp=time.time(),
        )

    return payload


@version_app.callback(invoke_without_command=True)
def version(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", "--log-level", help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux version` command.

    This function orchestrates the version reporting process by validating
    flags and then using the shared `new_run_command` helper to build and
    emit the final payload.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output; the exit code is the
            primary indicator of the outcome.
        verbose (bool): If True, includes Python, platform, and timestamp
            details in the output payload.
        fmt (str): The output format, either "json" or "yaml". Defaults to "json".
        pretty (bool): If True, pretty-prints the output for human readability.
        debug (bool): If True, enables debug diagnostics, implying `verbose`
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    if ctx.invoked_subcommand:
        return

    DIContainer.current().resolve(Emitter)
    DIContainer.current().resolve(TelemetryProtocol)
    command = "version"
    validate_common_flags(fmt, command, quiet)

    effective, _, fmt_lower = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include),
        quiet=effective.quiet,
        verbose=effective.verbose_level > 0,
        fmt=fmt_lower,
        pretty=effective.pretty,
        log_level=effective.log_level,
    )
