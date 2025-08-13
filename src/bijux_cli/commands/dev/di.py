# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `dev di` subcommand for the Bijux CLI.

This module provides a developer-focused command to introspect the internal
Dependency Injection (DI) container. It outputs a graph of all registered
service and factory protocols, which is useful for debugging the application's
architecture and service resolution.

Output Contract:
    * Success: `{"factories": list, "services": list}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal internal error occurred (e.g., during serialization).
    * `2`: An invalid argument or environment setting was provided (e.g.,
      bad output path, unreadable config, invalid limit).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import platform
from typing import Any

import typer
import yaml

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer

QUIET_OPTION = typer.Option(False, "-q", "--quiet", help=HELP_QUIET)
VERBOSE_OPTION = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE)
FORMAT_OPTION = typer.Option("json", "-f", "--format", help=HELP_FORMAT)
PRETTY_OPTION = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY)
DEBUG_OPTION = typer.Option(False, "-d", "--debug", help=HELP_DEBUG)
OUTPUT_OPTION = typer.Option(
    None,
    "-o",
    "--output",
    help="Write result to file(s). May be provided multiple times.",
)


def _key_to_name(key: object) -> str:
    """Converts a DI container key to its string name for serialization.

    Args:
        key (object): The key to convert, typically a class type or string.

    Returns:
        str: The string representation of the key.
    """
    if isinstance(key, str):
        return key
    name = getattr(key, "__name__", None)
    return str(name) if name else str(key)


def _build_dev_di_payload(include_runtime: bool) -> dict[str, Any]:
    """Builds the DI graph payload for structured output.

    Args:
        include_runtime (bool): If True, includes Python and platform runtime
            metadata in the payload.

    Returns:
        dict[str, Any]: A dictionary containing lists of registered 'factories'
            and 'services', along with optional runtime information.
    """
    di = DIContainer.current()

    factories = [
        {"protocol": _key_to_name(protocol), "alias": alias}
        for protocol, alias in di.factories()
    ]
    services = [
        {"protocol": _key_to_name(protocol), "alias": alias, "implementation": None}
        for protocol, alias in di.services()
    ]

    payload: dict[str, Any] = {"factories": factories, "services": services}
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


def dev_di_graph(
    quiet: bool = QUIET_OPTION,
    verbose: bool = VERBOSE_OPTION,
    fmt: str = FORMAT_OPTION,
    pretty: bool = PRETTY_OPTION,
    debug: bool = DEBUG_OPTION,
    output: list[Path] = OUTPUT_OPTION,
) -> None:
    """Generates and outputs the Dependency Injection (DI) container graph.

    This developer tool inspects the DI container, validates environment
    settings, and outputs the registration graph to stdout and/or one or more
    files.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.
        output (list[Path]): A list of file paths to write the output to.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "dev di"
    effective_include_runtime = (verbose or debug) and not quiet
    effective_pretty = True if (debug and not quiet) else pretty

    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=effective_include_runtime,
    )

    limit_env = os.environ.get("BIJUXCLI_DI_LIMIT")
    limit: int | None = None
    if limit_env is not None:
        try:
            limit = int(limit_env)
            if limit < 0:
                emit_error_and_exit(
                    f"Invalid BIJUXCLI_DI_LIMIT value: '{limit_env}'",
                    code=2,
                    failure="limit",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=effective_include_runtime,
                    debug=debug,
                )
        except (ValueError, TypeError):
            emit_error_and_exit(
                f"Invalid BIJUXCLI_DI_LIMIT value: '{limit_env}'",
                code=2,
                failure="limit",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective_include_runtime,
                debug=debug,
            )

    config_env = os.environ.get("BIJUXCLI_CONFIG")
    if config_env and not config_env.isascii():
        emit_error_and_exit(
            f"Config path contains non-ASCII characters: {config_env!r}",
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    if config_env:
        cfg_path = Path(config_env)
        if cfg_path.exists() and not os.access(cfg_path, os.R_OK):
            emit_error_and_exit(
                f"Config path not readable: {cfg_path}",
                code=2,
                failure="config_unreadable",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective_include_runtime,
                debug=debug,
            )

    try:
        payload = _build_dev_di_payload(effective_include_runtime)
        if limit is not None:
            payload["factories"] = payload["factories"][:limit]
            payload["services"] = payload["services"][:limit]
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    outputs = output
    if outputs:
        for p in outputs:
            if p.is_dir():
                emit_error_and_exit(
                    f"Output path is a directory: {p}",
                    code=2,
                    failure="output_dir",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=effective_include_runtime,
                    debug=debug,
                )
            p.parent.mkdir(parents=True, exist_ok=True)
            try:
                if fmt_lower == "json":
                    p.write_text(
                        json.dumps(payload, indent=2 if effective_pretty else None)
                        + "\n",
                        encoding="utf-8",
                    )
                else:
                    p.write_text(
                        yaml.safe_dump(
                            payload,
                            default_flow_style=False,
                            indent=2 if effective_pretty else None,
                        ),
                        encoding="utf-8",
                    )
            except OSError as exc:
                emit_error_and_exit(
                    f"Failed to write output file '{p}': {exc}",
                    code=2,
                    failure="output_write",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=effective_include_runtime,
                    debug=debug,
                )

        if quiet:
            raise typer.Exit(0)

    if os.environ.get("BIJUXCLI_TEST_FORCE_SERIALIZE_FAIL") == "1":
        emit_error_and_exit(
            "Forced serialization failure",
            code=1,
            failure="serialize",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda _: payload,
        quiet=quiet,
        verbose=effective_include_runtime,
        fmt=fmt_lower,
        pretty=effective_pretty,
        debug=debug,
    )
