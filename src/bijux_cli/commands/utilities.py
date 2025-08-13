# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides shared, reusable utilities for Bijux CLI commands.

This module centralizes common logic to ensure consistency and reduce code
duplication across the various command implementations. It includes a suite of
functions for handling standard CLI tasks, such as:

* **Validation:** Functions for validating common CLI flags (like `--format`)
    and checking the environment for non-ASCII characters or malformed
    configuration files.
* **Output & Exit:** A set of high-level emitters (`emit_and_exit`,
    `emit_error_and_exit`) that handle payload serialization (JSON/YAML),
    pretty-printing, and terminating the application with a contract-compliant
    exit code and structured message.
* **Command Orchestration:** A primary helper (`new_run_command`) that
    encapsulates the standard lifecycle of a command: validation, payload
    construction, and emission.
* **Parsing & Sanitization:** Helpers for sanitizing strings to be ASCII-safe
    and a pre-parser for global flags (`--quiet`, `--debug`, etc.) that
    operates before Typer's main dispatch.
* **Plugin Management:** Utilities for discovering and listing installed
    plugins from the filesystem.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from contextlib import suppress
import json
import os
from pathlib import Path
import platform
import re
import sys
import time
from typing import Any, NoReturn

import yaml

from bijux_cli.core.enums import OutputFormat
from bijux_cli.services.plugins import get_plugins_dir

_ALLOWED_CTRL = {"\n", "\r", "\t"}
_ENV_LINE_RX = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=[A-Za-z0-9_./\-]*$")
KNOWN = {
    "-h",
    "--help",
    "-q",
    "--quiet",
    "--debug",
    "-v",
    "--verbose",
    "-f",
    "--format",
    "--pretty",
    "--no-pretty",
}


def ascii_safe(text: Any, _field: str = "") -> str:
    """Converts any value to a string containing only printable ASCII characters.

    Non-ASCII characters are replaced with '?'. Newlines, carriage returns,
    and tabs are preserved.

    Args:
        text (Any): The value to sanitize.
        _field (str, optional): An unused parameter for potential future use
            in context or telemetry. Defaults to "".

    Returns:
        str: An ASCII-safe string.
    """
    text_str = text if isinstance(text, str) else str(text)

    return "".join(
        ch if (32 <= ord(ch) <= 126) or ch in _ALLOWED_CTRL else "?" for ch in text_str
    )


def normalize_format(fmt: str | None) -> str:
    """Normalizes a format string to lowercase and removes whitespace.

    Args:
        fmt (str | None): The format string to normalize.

    Returns:
        str: The normalized format string, or an empty string if input is None.
    """
    return (fmt or "").strip().lower()


def contains_non_ascii_env() -> bool:
    """Checks for non-ASCII characters in the CLI's environment.

    This function returns True if any of the following are detected:
    * The `BIJUXCLI_CONFIG` environment variable contains non-ASCII characters.
    * The file path pointed to by `BIJUXCLI_CONFIG` exists and its contents
        cannot be decoded as ASCII.
    * Any environment variable with a name starting with `BIJUXCLI_` has a
        value containing non-ASCII characters.

    Returns:
        bool: True if a non-ASCII condition is found, otherwise False.
    """
    config_path_str = os.environ.get("BIJUXCLI_CONFIG")
    if config_path_str:
        if not config_path_str.isascii():
            return True
        config_path = Path(config_path_str)
        if config_path.exists():
            try:
                config_path.read_text(encoding="ascii")
            except UnicodeDecodeError:
                return True
            except (IsADirectoryError, PermissionError, FileNotFoundError, OSError):
                pass

    for k, v in os.environ.items():
        if k.startswith("BIJUXCLI_") and not v.isascii():
            return True
    return False


def validate_common_flags(
    fmt: str,
    command: str,
    quiet: bool,
    include_runtime: bool = False,
) -> str:
    """Validates common CLI flags and environment settings.

    This function ensures the format is supported and the environment is
    ASCII-safe, exiting with a structured error if validation fails.

    Args:
        fmt (str): The requested output format.
        command (str): The name of the command for error reporting context.
        quiet (bool): If True, suppresses output on error before exiting.
        include_runtime (bool): If True, includes runtime info in error payloads.

    Returns:
        str: The validated and normalized format string ("json" or "yaml").

    Raises:
        SystemExit: Exits with code 2 for an unsupported format or 3 for
            a non-ASCII environment.
    """
    format_lower = (fmt or "").lower()
    if format_lower not in ("json", "yaml"):
        emit_error_and_exit(
            f"Unsupported format: {fmt}",
            code=2,
            failure="format",
            command=command,
            fmt=format_lower or "json",
            quiet=quiet,
            include_runtime=include_runtime,
            debug=False,
        )

    if contains_non_ascii_env():
        emit_error_and_exit(
            "Non-ASCII in configuration or environment",
            code=3,
            failure="ascii",
            command=command,
            fmt=format_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=False,
        )

    return format_lower


def validate_env_file_if_present(path_str: str) -> None:
    """Validates the syntax of an environment configuration file if it exists.

    Checks that every non-comment, non-blank line conforms to a `KEY=VALUE`
    pattern.

    Args:
        path_str (str): The path to the environment file.

    Raises:
        ValueError: If the file cannot be read or contains a malformed line.
    """
    if not path_str or not Path(path_str).exists():
        return
    try:
        text = Path(path_str).read_text(encoding="utf-8", errors="strict")
    except Exception as exc:
        raise ValueError(f"Cannot read config file: {exc}") from exc

    for i, line in enumerate(text.splitlines(), start=1):
        s = line.strip()
        if s and not s.startswith("#") and not _ENV_LINE_RX.match(s):
            raise ValueError(f"Malformed line {i} in config: {line!r}")


def new_run_command(
    command_name: str,
    payload_builder: Callable[[bool], Mapping[str, object]],
    quiet: bool,
    verbose: bool,
    fmt: str,
    pretty: bool,
    debug: bool,
    exit_code: int = 0,
) -> NoReturn:
    """Orchestrates the standard execution flow of a CLI command.

    This function handles dependency resolution, validation, payload
    construction, and final emission, ensuring a consistent lifecycle for all
    commands that use it.

    Args:
        command_name (str): The name of the command for telemetry/error context.
        payload_builder: A function that takes a boolean `include_runtime` and
            returns the command's structured output payload.
        quiet (bool): If True, suppresses normal output.
        verbose (bool): If True, includes runtime metadata in the output.
        fmt (str): The output format ("json" or "yaml").
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug-level output.
        exit_code (int): The exit code to use on successful execution.

    Raises:
        SystemExit: Always exits the process with the given `exit_code` or an
            appropriate error code on failure.
    """
    from bijux_cli.contracts import EmitterProtocol, TelemetryProtocol
    from bijux_cli.core.di import DIContainer

    DIContainer.current().resolve(EmitterProtocol)
    DIContainer.current().resolve(TelemetryProtocol)

    include_runtime = verbose or debug

    format_lower = validate_common_flags(
        fmt,
        command_name,
        quiet,
        include_runtime=include_runtime,
    )

    output_format = OutputFormat.YAML if format_lower == "yaml" else OutputFormat.JSON
    effective_pretty = debug or pretty

    try:
        payload = payload_builder(include_runtime)
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command_name,
            fmt=output_format,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )
    else:
        emit_and_exit(
            payload=payload,
            fmt=output_format,
            effective_pretty=effective_pretty,
            verbose=verbose,
            debug=debug,
            quiet=quiet,
            command=command_name,
            exit_code=exit_code,
        )


def emit_and_exit(
    payload: Mapping[str, Any],
    fmt: OutputFormat,
    effective_pretty: bool,
    verbose: bool,
    debug: bool,
    quiet: bool,
    command: str,
    *,
    exit_code: int = 0,
) -> NoReturn:
    """Serializes and emits a payload, records history, and exits.

    Args:
        payload (Mapping[str, Any]): The data to serialize and print.
        fmt (OutputFormat): The output format (JSON or YAML).
        effective_pretty (bool): If True, pretty-prints the output.
        verbose (bool): If True, includes runtime info in history records.
        debug (bool): If True, emits a diagnostic message to stderr.
        quiet (bool): If True, suppresses all output and exits immediately.
        command (str): The command name, used for history tracking.
        exit_code (int): The exit status code to use.

    Raises:
        SystemExit: Always exits the process with `exit_code`.
    """
    if (not quiet) and (not command.startswith("history")):
        try:
            from bijux_cli.contracts import HistoryProtocol
            from bijux_cli.core.di import DIContainer

            hist = DIContainer.current().resolve(HistoryProtocol)
            hist.add(
                command=command,
                params=[],
                success=(exit_code == 0),
                return_code=exit_code,
                duration_ms=0.0,
            )
        except PermissionError as exc:
            print(f"Permission denied writing history: {exc}", file=sys.stderr)
        except OSError as exc:
            import errno as _errno

            if exc.errno in (_errno.EACCES, _errno.EPERM):
                print(f"Permission denied writing history: {exc}", file=sys.stderr)
            elif exc.errno in (_errno.ENOSPC, _errno.EDQUOT):
                print(
                    f"No space left on device while writing history: {exc}",
                    file=sys.stderr,
                )
            else:
                print(f"Error writing history: {exc}", file=sys.stderr)
        except Exception as exc:
            print(f"Error writing history: {exc}", file=sys.stderr)

    if quiet:
        sys.exit(exit_code)

    if debug:
        print("Diagnostics: emitted payload", file=sys.stderr)

    indent = 2 if effective_pretty else None
    if fmt == OutputFormat.JSON:
        separators = (", ", ": ") if effective_pretty else (",", ":")
        output = json.dumps(payload, indent=indent, separators=separators)
    else:
        default_flow_style = None if effective_pretty else True
        output = yaml.safe_dump(
            payload,
            indent=indent,
            sort_keys=False,
            default_flow_style=default_flow_style,
        )
    cleaned = output.rstrip("\n")
    print(cleaned)
    sys.exit(exit_code)


def emit_error_and_exit(
    message: str,
    code: int,
    failure: str,
    command: str | None = None,
    fmt: str | None = None,
    quiet: bool = False,
    include_runtime: bool = False,
    debug: bool = False,
    extra: dict[str, Any] | None = None,
) -> NoReturn:
    """Emits a structured error payload to stderr and exits the process.

    Args:
        message (str): The primary error message.
        code (int): The exit status code.
        failure (str): A short, machine-readable failure code.
        command (str | None): The command name where the error occurred.
        fmt (str | None): The output format context.
        quiet (bool): If True, suppresses all output and exits immediately.
        include_runtime (bool): If True, adds runtime info to the error payload.
        debug (bool): If True, prints a full traceback to stderr.
        extra (dict[str, Any] | None): Additional fields to merge into the payload.

    Raises:
        SystemExit: Always exits the process with the specified `code`.
    """
    if quiet:
        sys.exit(code)

    if debug:
        import traceback

        traceback.print_exc(file=sys.stderr)

    error_payload = {"error": message, "code": code}
    if failure:
        error_payload["failure"] = failure
    if command:
        error_payload["command"] = command
    if fmt:
        error_payload["fmt"] = fmt
    if extra:
        error_payload.update(extra)
    if include_runtime:
        error_payload["python"] = ascii_safe(sys.version.split()[0], "python_version")
        error_payload["platform"] = ascii_safe(platform.platform(), "platform")
        error_payload["timestamp"] = str(time.time())

    try:
        output = json.dumps(error_payload).rstrip("\n")
        print(output, file=sys.stderr, flush=True)
    except Exception:
        print('{"error": "Unserializable error"}', file=sys.stderr, flush=True)
    sys.exit(code)


def parse_global_flags() -> dict[str, Any]:
    """Parses global CLI flags from `sys.argv` before Typer dispatch.

    This function inspects and consumes known global flags, rewriting `sys.argv`
    to contain only the remaining arguments. This allows global settings to be
    processed independently of the command-specific parsing done by Typer.

    Returns:
        dict[str, Any]: A dictionary of parsed flag values, such as `help`,
            `quiet`, `debug`, `verbose`, `format`, and `pretty`.

    Raises:
        SystemExit: If a flag requires an argument that is missing (e.g.,
            `--format` with no value).
    """
    argv = sys.argv[1:]
    flags: dict[str, Any] = {
        "help": False,
        "quiet": False,
        "debug": False,
        "verbose": False,
        "format": "json",
        "pretty": True,
    }
    retained: list[str] = []

    def _bail(msg: str, failure: str) -> NoReturn:
        """Emits a standardized error and exits with code 2.

        Args:
            msg (str): The error message to report.
            failure (str): A short failure code (e.g., "missing_argument").

        Raises:
            SystemExit: Always exits the process.
        """
        emit_error_and_exit(
            msg,
            code=2,
            failure=failure,
            command="global",
            fmt=flags["format"],
            quiet=flags["quiet"],
            include_runtime=flags["verbose"],
            debug=flags["debug"],
        )

    i = 0
    while i < len(argv):
        tok = argv[i]

        if tok in ("-h", "--help"):
            flags["help"] = True
            retained.append(tok)
            i += 1
        elif tok in ("-q", "--quiet"):
            flags["quiet"] = True
            i += 1
        elif tok == "--debug":
            flags["debug"] = True
            flags["verbose"] = True
            flags["pretty"] = True
            i += 1
        elif tok in ("-v", "--verbose"):
            flags["verbose"] = True
            i += 1
        elif tok == "--pretty":
            flags["pretty"] = True
            i += 1
        elif tok == "--no-pretty":
            flags["pretty"] = False
            i += 1
        elif tok in ("-f", "--format"):
            i += 1
            if i >= len(argv):
                _bail("Missing argument for --format", "missing_argument")
            else:
                value = argv[i].lower()
                flags["format"] = value
                if flags["help"]:
                    retained.append(tok.lstrip("-"))
                    retained.append(argv[i])
                if not flags["help"] and value not in ("json", "yaml"):
                    _bail(f"Unsupported format: {value}", "invalid_format")
                i += 1
        else:
            retained.append(tok)
            i += 1

    if flags["help"]:
        retained = [
            arg.lstrip("-") if arg.startswith("-") and arg not in KNOWN else arg
            for arg in retained
        ]

    sys.argv = [sys.argv[0], *retained]
    return flags


def list_installed_plugins() -> list[str]:
    """Scans the plugins directory and returns a list of installed plugin names.

    A directory is considered a valid plugin if it is a direct child of the
    plugins directory and contains a `plugin.py` file.

    Returns:
        list[str]: A sorted list of valid plugin names.

    Raises:
        RuntimeError: If the plugins directory is invalid, inaccessible,
            is not a directory, or contains a symlink loop.
    """
    plugins_dir = get_plugins_dir()

    try:
        resolved = plugins_dir.resolve(strict=True)
    except FileNotFoundError:
        return []
    except RuntimeError as e:
        raise RuntimeError(f"Symlink loop detected at '{plugins_dir}'.") from e
    except Exception as exc:
        raise RuntimeError(
            f"Plugins directory '{plugins_dir}' invalid or inaccessible."
        ) from exc

    if not resolved.is_dir():
        raise RuntimeError(f"Plugins directory '{plugins_dir}' is not a directory.")

    plugins: list[str] = []
    for entry in resolved.iterdir():
        with suppress(Exception):
            p = entry.resolve()
            if p.is_dir() and (p / "plugin.py").is_file():
                plugins.append(entry.name)

    plugins.sort()
    return plugins


def handle_list_plugins(
    command: str,
    quiet: bool,
    verbose: bool,
    fmt: str,
    pretty: bool,
    debug: bool,
) -> None:
    """Handles the logic for commands that list installed plugins.

    This function serves as a common handler for `plugins list` and similar
    commands. It retrieves the list of plugins and uses `new_run_command`
    to emit the result.

    Args:
        command (str): The name of the command being executed.
        quiet (bool): If True, suppresses normal output.
        verbose (bool): If True, includes runtime metadata in the payload.
        fmt (str): The requested output format ("json" or "yaml").
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug mode.

    Returns:
        None:
    """
    format_lower = validate_common_flags(fmt, command, quiet)

    try:
        plugins = list_installed_plugins()
    except RuntimeError as exc:
        emit_error_and_exit(
            str(exc),
            code=1,
            failure="dir_error",
            command=command,
            fmt=format_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )
    else:

        def _build_payload(include: bool) -> dict[str, object]:
            """Constructs a payload describing installed plugins.

            Args:
                include (bool): If True, includes Python/platform info.

            Returns:
                dict[str, object]: A dictionary containing a "plugins" list
                    and optional runtime metadata.
            """
            payload: dict[str, object] = {"plugins": plugins}
            if include:
                payload["python"] = ascii_safe(
                    platform.python_version(), "python_version"
                )
                payload["platform"] = ascii_safe(platform.platform(), "platform")
            return payload

        new_run_command(
            command_name=command,
            payload_builder=_build_payload,
            quiet=quiet,
            verbose=verbose,
            fmt=format_lower,
            pretty=pretty,
            debug=debug,
        )


__all__ = [
    "handle_list_plugins",
    "list_installed_plugins",
    "parse_global_flags",
    "emit_error_and_exit",
    "emit_and_exit",
    "new_run_command",
    "validate_env_file_if_present",
    "validate_common_flags",
    "contains_non_ascii_env",
    "normalize_format",
    "ascii_safe",
]
