# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `dev di` subcommand for the Bijux CLI.

This module provides a developer-focused command to introspect the internal
Dependency Injection (DI) container. It outputs a graph of all registered
service and factory protocols, which is useful for debugging the application's
architecture and service resolution.

Output Contract:
    * Success: `{"factories": list, "services": list}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal internal error occurred (e.g., during serialization).
    * `2`: An invalid argument or environment setting was provided (e.g.,
      bad output path, unreadable config, invalid limit).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

import os
from pathlib import Path
import platform
from typing import Any

import typer

from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
    normalize_format,
    raise_exit_intent,
    record_history,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    ENV_CONFIG,
    ENV_DI_LIMIT,
    ENV_TEST_FORCE_SERIALIZE_FAIL,
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
from bijux_cli.core.enums import ErrorType, ExitCode, OutputFormat
from bijux_cli.core.precedence import current_execution_policy

QUIET_OPTION = typer.Option(False, *OPT_QUIET, help=HELP_QUIET)
FORMAT_OPTION = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT)
PRETTY_OPTION = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY)
LOG_LEVEL_OPTION = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL)
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
    fmt: str = FORMAT_OPTION,
    pretty: bool = PRETTY_OPTION,
    log_level: str = LOG_LEVEL_OPTION,
    output: list[Path] = OUTPUT_OPTION,
) -> None:
    """Generates and outputs the Dependency Injection (DI) container graph.

    This developer tool inspects the DI container, validates environment
    settings, and outputs the registration graph to stdout and/or one or more
    files.

    Args:
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): The requested logging level.
        output (list[Path]): A list of file paths to write the output to.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "dev di"
    policy = current_execution_policy()
    quiet = policy.quiet
    log_level_value = policy.log_level
    effective_include_runtime = policy.include_runtime
    effective_pretty = policy.pretty
    fmt_lower = normalize_format(fmt) or OutputFormat.JSON

    limit_env = os.environ.get(ENV_DI_LIMIT)
    limit: int | None = None
    if limit_env is not None:
        try:
            limit = int(limit_env)
            if limit < 0:
                raise_exit_intent(
                    f"Invalid {ENV_DI_LIMIT} value: '{limit_env}'",
                    code=2,
                    failure="limit",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=effective_include_runtime,
                    log_level=log_level_value,
                    error_type=ErrorType.USER_INPUT,
                )
        except (ValueError, TypeError):
            raise_exit_intent(
                f"Invalid {ENV_DI_LIMIT} value: '{limit_env}'",
                code=2,
                failure="limit",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective_include_runtime,
                log_level=log_level_value,
                error_type=ErrorType.USER_INPUT,
            )

    config_env = os.environ.get(ENV_CONFIG)
    if config_env and not config_env.isascii():
        raise_exit_intent(
            f"Config path contains non-ASCII characters: {config_env!r}",
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
            error_type=ErrorType.ASCII,
        )

    if config_env:
        cfg_path = Path(config_env)
        if cfg_path.exists() and not os.access(cfg_path, os.R_OK):
            raise_exit_intent(
                f"Config path not readable: {cfg_path}",
                code=2,
                failure="config_unreadable",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=effective_include_runtime,
                log_level=log_level_value,
                error_type=ErrorType.USER_INPUT,
            )

    validate_common_flags(
        fmt_lower,
        command,
        quiet,
        include_runtime=effective_include_runtime,
        log_level=log_level_value,
    )

    try:
        payload = _build_dev_di_payload(effective_include_runtime)
        if limit is not None:
            payload = {
                "factories": payload["factories"][:limit],
                "services": payload["services"][:limit],
                **(
                    {}
                    if payload.get("python") is None
                    else {"python": payload["python"]}
                ),
                **(
                    {}
                    if payload.get("platform") is None
                    else {"platform": payload["platform"]}
                ),
            }
    except ValueError as exc:
        raise_exit_intent(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
            error_type=ErrorType.ASCII,
        )

    outputs = output
    if outputs:
        for p in outputs:
            if p.is_dir():
                raise_exit_intent(
                    f"Output path is a directory: {p}",
                    code=2,
                    failure="output_dir",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=effective_include_runtime,
                    log_level=log_level_value,
                    error_type=ErrorType.USER_INPUT,
                )
            p.parent.mkdir(parents=True, exist_ok=True)
            try:
                from bijux_cli.cli.core.command import resolve_serializer

                rendered = resolve_serializer().dumps(
                    payload, fmt=fmt_lower, pretty=effective_pretty
                )
                p.write_text(rendered.rstrip("\n") + "\n", encoding="utf-8")
            except OSError as exc:
                raise_exit_intent(
                    f"Failed to write output file '{p}': {exc}",
                    code=2,
                    failure="output_write",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=effective_include_runtime,
                    log_level=log_level_value,
                    error_type=ErrorType.USER_INPUT,
                )

        emit_output = not quiet
        if not emit_output:
            record_history(command, 0)
            from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError

            raise ExitIntentError(
                ExitIntent(
                    code=ExitCode.SUCCESS,
                    stream=None,
                    payload=None,
                    fmt=fmt_lower,
                    pretty=effective_pretty,
                    show_traceback=False,
                )
            )

    if os.environ.get(ENV_TEST_FORCE_SERIALIZE_FAIL) == "1":
        raise_exit_intent(
            "Forced serialization failure",
            code=1,
            failure="serialize",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
            error_type=ErrorType.INTERNAL,
        )

    new_run_command(
        command_name=command,
        payload_builder=lambda _: payload,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=effective_pretty,
        log_level=log_level,
    )
