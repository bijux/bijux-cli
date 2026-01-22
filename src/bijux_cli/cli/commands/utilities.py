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
    and a pre-parser for global flags (`--quiet`, `--log-level`, etc.) that
    operates before Typer's main dispatch.
* **Plugin Management:** Utilities for discovering and listing installed
    plugins from the filesystem.
"""

from __future__ import annotations

from collections.abc import Callable, Mapping
from contextlib import suppress
import json
import logging
import os
from pathlib import Path
import platform
import re
import sys
import time
from typing import Any, NoReturn

from bijux_cli.core.contracts import Serializer
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.precedence import EffectiveConfig
from bijux_cli.plugins import get_plugins_dir

_ALLOWED_CTRL = {"\n", "\r", "\t"}
_ENV_LINE_RX = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=[A-Za-z0-9_./\-]*$")
KNOWN = {
    "-h",
    "--help",
    "-q",
    "--quiet",
    "-v",
    "--verbose",
    "-f",
    "--format",
    "--log-level",
    "--pretty",
    "--no-pretty",
}


def effective_defaults() -> dict[str, Any]:
    """Fetch defaults from the bootstrapped EffectiveConfig when available."""
    try:
        from bijux_cli.app.di import DIContainer
        from bijux_cli.core.precedence import EffectiveConfig

        effective = DIContainer.current().resolve(EffectiveConfig)
        if not isinstance(effective, EffectiveConfig):
            raise TypeError("EffectiveConfig not available")
    except Exception:
        return {
            "quiet": False,
            "verbose": False,
            "pretty": True,
            "log_level": "info",
            "color": "auto",
            "format": "json",
            "json": False,
        }
    return {
        "quiet": effective.quiet,
        "verbose": effective.verbose_level > 0,
        "pretty": effective.pretty,
        "log_level": effective.log_level,
        "color": effective.color,
        "format": effective.fmt,
        "json": effective.json,
    }


def resolve_serializer() -> Serializer:
    """Resolve the serializer adapter from the DI container or fallback."""
    try:
        from bijux_cli.app.di import DIContainer

        serializer = DIContainer.current().resolve(Serializer)
        if hasattr(serializer, "dumps"):
            return serializer
    except Exception as exc:
        logging.getLogger(__name__).debug("Failed to resolve serializer", exc_info=exc)

    class _FallbackSerializer:
        def dumps(self, obj: Any, *, fmt: Any = "json", pretty: bool = False) -> str:
            if str(fmt).lower() == "yaml":
                try:
                    import yaml

                    return yaml.safe_dump(obj, sort_keys=False)
                except Exception:
                    return json.dumps(obj, indent=2 if pretty else None)
            return json.dumps(obj, indent=2 if pretty else None)

        def dumps_bytes(
            self, obj: Any, *, fmt: Any = "json", pretty: bool = False
        ) -> bytes:
            return self.dumps(obj, fmt=fmt, pretty=pretty).encode("utf-8")

        def loads(
            self, data: str | bytes, *, fmt: Any = "json", pretty: bool = False
        ) -> Any:
            _ = pretty
            if str(fmt).lower() == "yaml":
                try:
                    import yaml

                    return yaml.safe_load(data)
                except Exception:
                    return data
            return json.loads(data)

        def emit(
            self, payload: Any, *, fmt: Any = "json", pretty: bool = False
        ) -> None:
            sys.stdout.write(self.dumps(payload, fmt=fmt, pretty=pretty))
            sys.stdout.write("\n")

    return _FallbackSerializer()


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
        try:
            config_path = Path(config_path_str)
        except NotImplementedError:
            return False
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
    from bijux_cli.core.precedence import resolve_effective_config

    resolved = resolve_effective_config(
        cli={"format": fmt},
        env={},
        file={},
        defaults=effective_defaults(),
    )
    format_lower = resolved.fmt
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
    log_level: str,
    exit_code: int = 0,
) -> None:
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
        log_level (str): The requested log level.
        exit_code (int): The exit code to use on successful execution.

    Raises:
        SystemExit: Always exits the process with the given `exit_code` or an
            appropriate error code on failure.
    """
    from bijux_cli.app.di import DIContainer
    from bijux_cli.core.contracts import Emitter
    from bijux_cli.services.contracts import TelemetryProtocol

    DIContainer.current().resolve(Emitter)
    DIContainer.current().resolve(TelemetryProtocol)

    from bijux_cli.core.precedence import resolve_effective_config

    resolved = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "pretty": pretty,
            "format": fmt,
            "log_level": log_level,
        },
        env={},
        file={},
        defaults=effective_defaults(),
    )
    include_runtime = resolved.include_runtime

    format_lower = validate_common_flags(
        resolved.fmt,
        command_name,
        resolved.quiet,
        include_runtime=include_runtime,
    )

    output_format = OutputFormat.YAML if format_lower == "yaml" else OutputFormat.JSON
    effective_pretty = resolved.pretty

    try:
        payload = payload_builder(include_runtime)
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command_name,
            fmt=output_format,
            quiet=resolved.quiet,
            include_runtime=include_runtime,
            debug=(resolved.log_level == "debug"),
        )
    else:
        emit_and_exit(
            payload=payload,
            fmt=output_format,
            effective_pretty=effective_pretty,
            verbose=resolved.verbose_level > 0,
            debug=(resolved.log_level == "debug"),
            quiet=resolved.quiet,
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
            from bijux_cli.app.di import DIContainer
            from bijux_cli.services.history.contracts import HistoryProtocol

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

    serializer = resolve_serializer()
    output = serializer.dumps(payload, fmt=fmt, pretty=effective_pretty)
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

    serializer = resolve_serializer()
    try:
        output = serializer.dumps(
            error_payload,
            fmt=str(error_payload.get("format", "json")),
            pretty=False,
        ).rstrip("\n")
        print(output, file=sys.stderr, flush=True)
    except Exception:
        print('{"error": "Unserializable error"}', file=sys.stderr, flush=True)
    sys.exit(code)


def parse_global_flags() -> dict[str, Any]:
    """Parses global CLI flags from `sys.argv` before Typer dispatch."""
    from bijux_cli.cli.flags import apply_parsed_flags
    from bijux_cli.cli.flags import parse_global_flags as _parse

    def _bail(msg: str, failure: str, flags: dict[str, Any]) -> None:
        emit_error_and_exit(
            msg,
            code=2,
            failure=failure,
            command="global",
            fmt=flags["format"],
            quiet=flags["quiet"],
            include_runtime=flags["verbose"],
            debug=str(flags["log_level"]).lower() == "debug",
        )

    flags, retained = _parse(sys.argv[1:], _bail)
    apply_parsed_flags(flags, retained)
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
    log_level: str,
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
        log_level (str): The requested logging level.

    Returns:
        None:
    """
    effective, output_format, format_lower = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )

    try:
        from bijux_cli.plugins.metadata import list_plugins

        plugins = list_plugins()
    except Exception as exc:
        emit_error_and_exit(
            str(exc),
            code=1,
            failure="dir_error",
            command=command,
            fmt=output_format,
            quiet=effective.quiet,
            include_runtime=effective.include_runtime,
            debug=effective.log_level == "debug",
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
            quiet=effective.quiet,
            verbose=effective.verbose_level > 0,
            fmt=format_lower,
            pretty=effective.pretty,
            log_level=effective.log_level,
        )


def resolve_command_config(
    *,
    command: str,
    quiet: bool,
    verbose: bool,
    log_level: str,
    fmt: str,
    pretty: bool,
) -> tuple[EffectiveConfig, OutputFormat, str]:
    """Resolve CLI flags into an effective config and validated output format."""
    from bijux_cli.core.precedence import resolve_effective_config

    effective = resolve_effective_config(
        cli={
            "quiet": quiet,
            "verbose": verbose,
            "pretty": pretty,
            "format": fmt,
            "log_level": log_level,
        },
        env={},
        file={},
        defaults={
            "quiet": False,
            "verbose": False,
            "pretty": True,
            "log_level": "info",
            "color": "auto",
            "format": "json",
            "json": False,
        },
    )
    format_lower = validate_common_flags(
        effective.fmt,
        command,
        effective.quiet,
        include_runtime=effective.include_runtime,
    )
    output_format = OutputFormat.YAML if format_lower == "yaml" else OutputFormat.JSON
    return effective, output_format, format_lower


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
    "resolve_serializer",
]
