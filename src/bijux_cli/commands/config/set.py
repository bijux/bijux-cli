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
import os
import platform
import re
import string
import sys

import typer

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    parse_global_flags,
)
from bijux_cli.contracts import ConfigProtocol
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer


def set_config(
    ctx: typer.Context,
    pair: str | None = typer.Argument(
        None, help="KEY=VALUE to set; if omitted, read from stdin"
    ),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
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
    cfg_path = os.environ.get("BIJUXCLI_CONFIG", "") or ""
    if cfg_path:
        try:
            cfg_path.encode("ascii")
        except UnicodeEncodeError:
            emit_error_and_exit(
                "Non-ASCII characters in config path",
                code=3,
                failure="ascii",
                command="config set",
                fmt="json",
                quiet=False,
                include_runtime=False,
                debug=False,
                extra={"path": "[non-ascii path provided]"},
            )
    flags = parse_global_flags()
    quiet = flags["quiet"]
    verbose = flags["verbose"]
    fmt = flags["format"]
    pretty = flags["pretty"]
    debug = flags["debug"]
    include_runtime = verbose
    fmt_lower = fmt.lower()
    command = "config set"
    if os.name == "posix":
        with suppress(Exception):
            import fcntl

            with open(cfg_path, "a+") as fh:
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
    if pair is None:
        if sys.stdin.isatty():
            emit_error_and_exit(
                "Missing argument: KEY=VALUE required",
                code=2,
                failure="missing_argument",
                command=command,
                fmt=fmt_lower,
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
            fmt=fmt_lower,
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
            fmt=fmt_lower,
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
            fmt=fmt_lower,
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
            fmt=fmt_lower,
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
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
            extra={"key": key},
        )
    config_svc = DIContainer.current().resolve(ConfigProtocol)
    try:
        config_svc.set(key, service_value_str)
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

    def payload_builder(include_runtime: bool) -> dict[str, object]:
        """Builds the payload confirming a key was set or updated.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            dict[str, object]: The structured payload.
        """
        payload: dict[str, object] = {
            "status": "updated",
            "key": key,
            "value": service_value_str,
        }
        if include_runtime:
            payload["python"] = ascii_safe(platform.python_version(), "python_version")
            payload["platform"] = ascii_safe(platform.platform(), "platform")
        return payload

    new_run_command(
        command_name=command,
        payload_builder=payload_builder,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
