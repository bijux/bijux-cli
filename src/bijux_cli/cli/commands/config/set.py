# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `config set` subcommand for the Bijux CLI.

This module contains the logic for creating or updating a key-value pair in
the active configuration store. It accepts input either as a direct argument
or from stdin, performs strict validation on keys and values, and provides a
structured, machine-readable response.

Output Contract:
    * Success: `{"status": "updated", "key": str, "value": str}`
    * Verbose: Adds `{"python": str, "platform": str}` to the payload.
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

from bijux_cli.cli.commands.payloads import ConfigSetPayload
from bijux_cli.cli.constants import (
    ENV_CONFIG,
    HELP_FORMAT,
    HELP_LOG_LEVEL,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
    OPT_FORMAT,
    OPT_LOG_LEVEL,
    OPT_PRETTY,
    OPT_QUIET,
    OPT_VERBOSE,
)
from bijux_cli.cli.core.emit import emit_error_and_exit
from bijux_cli.cli.core.output import new_run_command, resolve_command_config
from bijux_cli.cli.core.validation import ascii_safe
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import LogLevel, OutputFormat
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
    debug: bool,
) -> ConfigSetIntent:
    """Parse and validate a KEY=VALUE pair for config set."""
    if pair is None:
        if sys.stdin.isatty():
            emit_error_and_exit(
                "Missing argument: KEY=VALUE required",
                code=2,
                failure="missing_argument",
                command=command,
                fmt=fmt,
                quiet=quiet,
                include_runtime=include_runtime,
                debug=debug,
            )
        pair = sys.stdin.read().rstrip("\n")
    if not pair or "=" not in pair:
        emit_error_and_exit(
            "Invalid argument: KEY=VALUE required",
            code=2,
            failure="invalid_argument",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
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
        emit_error_and_exit(
            "Key cannot be empty",
            code=2,
            failure="empty_key",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )
    if not all(ord(c) < 128 for c in key + service_value_str):
        emit_error_and_exit(
            "Non-ASCII characters are not allowed in keys or values.",
            code=3,
            failure="ascii_error",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            extra={"key": key},
        )
    if not re.match(r"^[A-Za-z0-9_]+$", key):
        emit_error_and_exit(
            "Invalid key: only alphanumerics and underscore allowed.",
            code=2,
            failure="invalid_key",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            extra={"key": key},
        )
    if not all(
        c in string.printable and c not in "\r\n\t\x0b\x0c" for c in service_value_str
    ):
        emit_error_and_exit(
            "Control characters are not allowed in config values.",
            code=3,
            failure="control_char_error",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            extra={"key": key},
        )
    return ConfigSetIntent(key=key, value=service_value_str)


def set_config(
    ctx: typer.Context,
    pair: str | None = typer.Argument(
        None, help="KEY=VALUE to set; if omitted, read from stdin"
    ),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
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
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format, "json" or "yaml".
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing the error.
    """
    cfg_path = os.environ.get(ENV_CONFIG, "") or ""
    if cfg_path:
        try:
            cfg_path.encode("ascii")
        except UnicodeEncodeError:
            emit_error_and_exit(
                "Non-ASCII characters in config path",
                code=3,
                failure="ascii",
                command="config set",
                fmt=OutputFormat.JSON,
                quiet=False,
                include_runtime=False,
                debug=False,
                extra={"path": "[non-ascii path provided]"},
            )
    command = "config set"
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
    include_runtime = effective.include_runtime
    if cfg_path:
        with suppress(Exception), open(cfg_path, "a+") as fh:
            try:
                fcntl.flock(fh, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError:
                emit_error_and_exit(
                    "Config file is locked",
                    code=1,
                    failure="file_locked",
                    command=command,
                    fmt=fmt_lower,
                    quiet=quiet,
                    include_runtime=include_runtime,
                    debug=debug,
                    extra={"path": cfg_path},
                )
            finally:
                with suppress(Exception):
                    fcntl.flock(fh, fcntl.LOCK_UN)
    intent = _parse_pair(
        pair,
        command=command,
        fmt=fmt_lower,
        quiet=quiet,
        include_runtime=include_runtime,
        debug=debug,
    )
    config_svc = DIContainer.current().resolve(ConfigProtocol)
    try:
        config_svc.set(intent.key, intent.value)
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to set config: {exc}",
            code=1,
            failure="set_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    def payload_builder(include_runtime: bool) -> ConfigSetPayload:
        """Builds the payload confirming a key was set or updated.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            ConfigSetPayload: The structured payload.
        """
        payload = ConfigSetPayload(status="updated", key=intent.key, value=intent.value)
        if include_runtime:
            return ConfigSetPayload(
                status=payload.status,
                key=payload.key,
                value=payload.value,
                python=ascii_safe(platform.python_version(), "python_version"),
                platform=ascii_safe(platform.platform(), "platform"),
            )
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
