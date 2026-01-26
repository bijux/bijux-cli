# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `version` command for the Bijux CLI.

This module reports the CLI's version and runtime environment information.
The output is machine-readable, available in JSON or YAML, and is designed
to be safe for automation and scripting by adhering to a strict output
contract and ASCII hygiene.

Output Contract:
    * Success: `{"version": str}`
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

from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    ENV_VERSION,
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
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.core.runtime import AsyncTyper
from bijux_cli.core.version import __version__ as cli_version
from bijux_cli.infra.contracts import Emitter
from bijux_cli.services.contracts import TelemetryProtocol

typer.core.rich = None  # type: ignore[attr-defined]

version_app = AsyncTyper(
    name="version",
    help="Show the CLI version.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)


def _build_payload(include_runtime: bool) -> dict[str, object]:
    """Builds the structured payload for the version command.

    The version can be overridden by a dedicated environment variable,
    which is validated for correctness.

    Args:
        include_runtime (bool): If True, appends Python/platform details
            and a timestamp to the payload.

    Returns:
        Mapping[str, object]: A dictionary containing the CLI version and
            optional runtime metadata.

    Raises:
        ValueError: If the override env var is set but is empty, too long,
            contains non-ASCII characters, or is not a valid semantic version.
    """
    version_env = os.environ.get(ENV_VERSION)
    if version_env is not None:
        if not (1 <= len(version_env) <= 1024):
            raise ValueError(f"{ENV_VERSION} is empty or too long")
        if not all(ord(c) < 128 for c in version_env):
            raise ValueError(f"{ENV_VERSION} contains non-ASCII")
        if not re.fullmatch(r"\d+\.\d+\.\d+", version_env):
            raise ValueError(f"{ENV_VERSION} is not valid semantic version (x.y.z)")
        version_ = version_env
    else:
        version_ = cli_version

    payload: dict[str, object] = {"version": ascii_safe(version_, ENV_VERSION)}
    if include_runtime:
        payload.update(
            {
                "python": ascii_safe(platform.python_version(), "python_version"),
                "platform": ascii_safe(platform.platform(), "platform"),
                "timestamp": time.time(),
            }
        )
    return payload


@version_app.callback(invoke_without_command=True)
def version(
    ctx: typer.Context,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux version` command.

    This function orchestrates the version reporting process by validating
    flags and then using the shared `new_run_command` helper to build and
    emit the final payload.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        quiet (bool): If True, suppresses all output; the exit code is the
            primary indicator of the outcome.
            details in the output payload.
        fmt (str): The output format, either "json" or "yaml". Defaults to "json".
        pretty (bool): If True, pretty-prints the output for human readability.        log_level (str): Logging level for diagnostics.
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

    effective = current_execution_policy()
    fmt_lower = validate_common_flags(
        fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
        log_level=effective.log_level,
    )

    new_run_command(
        command_name=command,
        payload_builder=lambda include: _build_payload(include),
        quiet=effective.quiet,
        fmt=fmt_lower,
        pretty=effective.pretty,
        log_level=effective.log_level,
    )
