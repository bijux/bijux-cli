# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `config set` subcommand for the Bijux CLI.

This module contains the logic for creating or updating a key-value pair in
the active configuration store. It accepts input either as a direct argument
or from stdin, performs strict validation on keys and values, and provides a
structured, machine-readable response.

Output Contract:
    * Success: `{"status": "updated", "key": str, "value": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: An unexpected error occurred, such as a file lock or write failure.
    * `2`: An invalid argument was provided (e.g., malformed pair, invalid key).
    * `3`: The key, value, or configuration path contained non-ASCII or forbidden
      control characters.
"""

from __future__ import annotations

from contextlib import suppress
from dataclasses import dataclass
import fcntl
import os
import platform
import re
import string
import sys

import typer

from bijux_cli.cli.core.command import (
    ascii_safe,
    new_run_command,
    raise_exit_intent,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    ENV_CONFIG,
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
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.services.config.contracts import ConfigProtocol


@dataclass(frozen=True)
class ConfigSetIntent:
    """Parsed intent for a config set operation."""

    key: str
    value: str


def _parse_pair(
    pair: str | None,
    *,
    command: str,
    fmt: OutputFormat,
    quiet: bool,
    include_runtime: bool,
    log_level: LogLevel,
) -> ConfigSetIntent:
    """Parse and validate a KEY=VALUE pair for config set."""
    if pair is None:
        if sys.stdin.isatty():
            raise_exit_intent(
                "Missing argument: KEY=VALUE required",
                code=2,
                failure="missing_argument",
                command=command,
                fmt=fmt,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level,
            )
        pair = sys.stdin.read().rstrip("\n")
    if not pair or "=" not in pair:
        raise_exit_intent(
            "Invalid argument: KEY=VALUE required",
            code=2,
            failure="invalid_argument",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )
    raw_key, raw_value = pair.split("=", 1)
    key = raw_key.strip()
    service_value_str = raw_value
    if len(service_value_str) >= 2 and (
        (service_value_str[0] == service_value_str[-1] == '"')
        or (service_value_str[0] == service_value_str[-1] == "'")
    ):
        import codecs

        service_value_str = codecs.decode(service_value_str[1:-1], "unicode_escape")
    if not key:
        raise_exit_intent(
            "Key cannot be empty",
            code=2,
            failure="empty_key",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )
    if not all(ord(c) < 128 for c in key + service_value_str):
        raise_exit_intent(
            "Non-ASCII characters are not allowed in keys or values.",
            code=3,
            failure="ascii_error",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            extra={"key": key},
        )
    if not re.match(r"^[A-Za-z0-9_]+$", key):
        raise_exit_intent(
            "Invalid key: only alphanumerics and underscore allowed.",
            code=2,
            failure="invalid_key",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            extra={"key": key},
            error_type=ErrorType.USER_INPUT,
        )
    if not all(
        c in string.printable and c not in "\r\n\t\x0b\x0c" for c in service_value_str
    ):
        raise_exit_intent(
            "Control characters are not allowed in config values.",
            code=3,
            failure="control_char_error",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            extra={"key": key},
        )
    return ConfigSetIntent(key=key, value=service_value_str)


def set_config(
    ctx: typer.Context,
    pair: str | None = typer.Argument(
        None, help="KEY=VALUE to set; if omitted, read from stdin"
    ),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Sets or updates a configuration key-value pair.

    This function orchestrates the `set` operation. It accepts a `KEY=VALUE`
    pair from either a command-line argument or standard input. It performs
    extensive validation on the key and value for format and content, handles
    file locking to prevent race conditions, and emits a structured payload
    confirming the update.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        pair (str | None): A string in "KEY=VALUE" format. If None, the pair
            is read from stdin.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    command = "config set"
    policy = current_execution_policy()
    quiet = policy.quiet
    include_runtime = policy.include_runtime
    pretty = policy.pretty
    log_level_value = policy.log_level
    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )
    cfg_path = os.environ.get(ENV_CONFIG, "") or ""
    if cfg_path:
        try:
            cfg_path.encode("ascii")
        except UnicodeEncodeError:
            raise_exit_intent(
                "Non-ASCII characters in config path",
                code=3,
                failure="ascii",
                command="config set",
                fmt=OutputFormat.JSON,
                quiet=False,
                include_runtime=False,
                extra={"path": "[non-ascii path provided]"},
                log_level=log_level_value,
                error_type=ErrorType.ASCII,
            )
    if cfg_path:
        try:
            with open(cfg_path, "a+") as fh:
                try:
                    fcntl.flock(fh, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except OSError:
                    raise_exit_intent(
                        "Config file is locked",
                        code=1,
                        failure="file_locked",
                        command=command,
                        fmt=fmt_lower,
                        quiet=quiet,
                        include_runtime=include_runtime,
                        log_level=log_level_value,
                        extra={"path": cfg_path},
                    )
                finally:
                    with suppress(Exception):
                        fcntl.flock(fh, fcntl.LOCK_UN)
        except OSError:
            pass
    intent = _parse_pair(
        pair,
        command=command,
        fmt=fmt_lower,
        quiet=quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )
    config_svc = DIContainer.current().resolve(ConfigProtocol)
    try:
        config_svc.set(intent.key, intent.value)
    except Exception as exc:
        raise_exit_intent(
            f"Failed to set config: {exc}",
            code=1,
            failure="set_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level_value,
        )

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds the payload confirming a key was set or updated."""
        payload: dict[str, object] = {
            "status": "updated",
            "key": intent.key,
            "value": intent.value,
        }
        if include_runtime:
            return {
                "status": payload["status"],
                "key": intent.key,
                "value": intent.value,
                "python": ascii_safe(platform.python_version(), "python_version"),
                "platform": ascii_safe(platform.platform(), "platform"),
            }
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level_value,
    )
