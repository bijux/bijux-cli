# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Provides the main entry point and lifecycle orchestration for the Bijux CLI.

This module is the primary entry point when the CLI is executed. It is
responsible for orchestrating the entire lifecycle of a command invocation,
from initial setup to final exit.

Key responsibilities include:
    * **Environment Setup:** Configures structured logging (`structlog`) and
        disables terminal colors for tests.
    * **Argument Pre-processing:** Cleans and validates command-line arguments
        before they are passed to the command parser.
    * **Service Initialization:** Initializes the dependency injection container,
        registers all default services, and starts the core `Engine`.
    * **Application Assembly:** Builds the main `Typer` application, including
        all commands and dynamic plugins.
    * **Execution and Error Handling:** Invokes the Typer application, catches
        all top-level exceptions (including `Typer` errors, custom `CommandError`
        exceptions, and `KeyboardInterrupt`), and translates them into
        structured error messages and standardized exit codes.
    * **History Recording:** Persists the command to the history service after
        execution.
"""

from __future__ import annotations

import contextlib
from contextlib import suppress
import importlib.metadata as importlib_metadata
import io
import json
import logging
import os
import sys
import time
from typing import IO, Any, AnyStr

import click
from click.exceptions import NoSuchOption, UsageError
import structlog
import typer

from bijux_cli.cli import build_app
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import OutputFormat
from bijux_cli.core.exceptions import CommandError
from bijux_cli.services import register_default_services
from bijux_cli.services.history import History

_orig_stderr = sys.stderr
_orig_click_echo = click.echo
_orig_click_secho = click.secho


class _FilteredStderr(io.TextIOBase):
    """A proxy for `sys.stderr` that filters a specific noisy plugin warning."""

    def write(self, data: str) -> int:
        """Writes data to stderr, suppressing a specific known warning.

        Args:
            data (str): The string to write.

        Returns:
            int: The number of characters written, or 0 if suppressed.
        """
        noise = "Plugin 'test-src' does not expose a Typer app via 'cli()' or 'app'"
        if noise in data:
            return 0

        if _orig_stderr.closed:
            return 0

        return _orig_stderr.write(data)

    def flush(self) -> None:
        """Flushes the underlying stderr stream."""
        if not _orig_stderr.closed:
            _orig_stderr.flush()

    def __getattr__(self, name: str) -> Any:
        """Delegates attribute access to the original `sys.stderr`.

        Args:
            name (str): The name of the attribute to access.

        Returns:
            Any: The attribute from the original `sys.stderr`.
        """
        return getattr(_orig_stderr, name)


sys.stderr = _FilteredStderr()


def _filtered_echo(
    message: Any = None,
    file: IO[AnyStr] | None = None,
    nl: bool = True,
    err: bool = False,
    color: bool | None = None,
    **styles: Any,
) -> None:
    """A replacement for `click.echo` that filters a known plugin warning.

    Args:
        message (Any): The message to print.
        file (IO[AnyStr] | None): The file to write to.
        nl (bool): If True, appends a newline.
        err (bool): If True, writes to stderr instead of stdout.
        color (bool | None): If True, enables color output.
        **styles (Any): Additional style arguments for colored output.

    Returns:
        None
    """
    text = "" if message is None else str(message)
    if (
        text.startswith("[WARN] Plugin 'test-src'")
        and "does not expose a Typer app" in text
    ):
        return

    if styles:
        _orig_click_secho(message, file=file, nl=nl, err=err, color=color, **styles)
    else:
        _orig_click_echo(message, file=file, nl=nl, err=err, color=color)


click.echo = _filtered_echo
click.secho = _filtered_echo
typer.echo = _filtered_echo
typer.secho = _filtered_echo


def disable_cli_colors_for_test() -> None:
    """Disables color output from various libraries for test environments.

    This function checks for the `BIJUXCLI_TEST_MODE` environment variable and,
    if set, attempts to disable color output to ensure clean, predictable
    test results.
    """
    if os.environ.get("BIJUXCLI_TEST_MODE") != "1":
        return
    os.environ["NO_COLOR"] = "1"
    try:
        from rich.console import Console

        Console().no_color = True
    except ImportError:
        pass
    try:
        import colorama  # pyright: ignore[reportMissingModuleSource]

        colorama.deinit()
    except ImportError:
        pass
    try:
        import prompt_toolkit

        prompt_toolkit.shortcuts.set_title = (
            lambda text: None  # pyright: ignore[reportUnknownLambdaType]
        )
    except ImportError:  # pragma: no cover
        pass


def should_record_command_history(command_line: list[str]) -> bool:
    """Determines whether the given command should be recorded in the history.

    History recording is disabled under the following conditions:
    * The `BIJUXCLI_DISABLE_HISTORY` environment variable is set to "1".
    * The command line is empty.
    * The command is "history" or "help".

    Args:
        command_line (list[str]): The list of command-line input tokens.

    Returns:
        bool: True if the command should be recorded, otherwise False.
    """
    if os.environ.get("BIJUXCLI_DISABLE_HISTORY") == "1":
        return False
    if not command_line:
        return False
    return command_line[0].lower() not in {"history", "help"}


def is_quiet_mode(args: list[str]) -> bool:
    """Checks if the CLI was invoked with a quiet flag.

    Args:
        args (list[str]): The list of command-line arguments.

    Returns:
        bool: True if `--quiet` or `-q` is present, otherwise False.
    """
    return any(arg in ("--quiet", "-q") for arg in args)


def print_json_error(msg: str, code: int = 2, quiet: bool = False) -> None:
    """Prints a structured JSON error message.

    The message is printed to stdout for usage errors (code 2) and stderr for
    all other errors, unless quiet mode is enabled.

    Args:
        msg (str): The error message.
        code (int): The error code to include in the JSON payload.
        quiet (bool): If True, suppresses all output.
    """
    if not quiet:
        print(
            json.dumps({"error": msg, "code": code}),
            file=sys.stdout if code == 2 else sys.stderr,
        )


def get_usage_for_args(args: list[str], app: typer.Typer) -> str:
    """Gets the CLI help message for a given set of arguments.

    This function simulates invoking the CLI with `--help` to capture the
    contextual help message without exiting the process.

    Args:
        args (list[str]): The CLI arguments leading up to the help flag.
        app (typer.Typer): The `Typer` application instance.

    Returns:
        str: The generated help/usage message.
    """
    from contextlib import redirect_stdout
    import io

    subcmds = []
    for arg in args:
        if arg in ("--help", "-h"):
            break
        subcmds.append(arg)

    with io.StringIO() as buf, redirect_stdout(buf):
        with suppress(SystemExit):
            app(subcmds + ["--help"], standalone_mode=False)
        return buf.getvalue()


def _strip_format_help(args: list[str]) -> list[str]:
    """Removes an ambiguous `--format --help` combination from arguments.

    This prevents a parsing error where `--help` could be interpreted as the
    value for the `--format` option.

    Args:
        args (list[str]): The original list of command-line arguments.

    Returns:
        list[str]: A filtered list of arguments.
    """
    new_args = []
    skip_next = False
    for i, arg in enumerate(args):
        if skip_next:
            skip_next = False
            continue
        if (
            arg in ("--format", "-f")
            and i + 1 < len(args)
            and args[i + 1] in ("--help", "-h")
        ):
            skip_next = True
            continue
        new_args.append(arg)
    return new_args


def check_missing_format_argument(args: list[str]) -> str | None:
    """Checks if a `--format` or `-f` flag is missing its required value.

    Args:
        args (list[str]): The list of command-line arguments.

    Returns:
        str | None: An error message if the value is missing, otherwise None.
    """
    for i, arg in enumerate(args):
        if arg in ("--format", "-f"):
            if i + 1 >= len(args):
                return "Option '--format' requires an argument"
            next_arg = args[i + 1]
            if next_arg.startswith("-"):
                return "Option '--format' requires an argument"
    return None


def setup_structlog(debug: bool = False) -> None:
    """Configures `structlog` for the application.

    Args:
        debug (bool): If True, configures human-readable console output at the
            DEBUG level. If False, configures JSON output at the CRITICAL level.
    """
    level = logging.DEBUG if debug else logging.CRITICAL
    logging.basicConfig(level=level, stream=sys.stderr, format="%(message)s")

    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.stdlib.add_log_level,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.UnicodeDecoder(),
            (
                structlog.dev.ConsoleRenderer()
                if debug
                else structlog.processors.JSONRenderer()
            ),
        ],
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )


def main() -> int:
    """The main entry point for the Bijux CLI.

    This function orchestrates the entire lifecycle of a CLI command, from
    argument parsing and setup to execution and history recording.

    Returns:
        int: The final exit code of the command.
            * `0`: Success.
            * `1`: A generic command error occurred.
            * `2`: A usage error or invalid option was provided.
            * `130`: The process was interrupted by the user (Ctrl+C).
    """
    args = _strip_format_help(sys.argv[1:])

    quiet = is_quiet_mode(args)
    if quiet:
        with contextlib.suppress(Exception):
            sys.stderr = open(os.devnull, "w")  # noqa: SIM115
    debug = "--debug" in sys.argv or os.environ.get("BIJUXCLI_DEBUG") == "1"
    setup_structlog(debug)
    disable_cli_colors_for_test()

    if any(a in ("--version", "-V") for a in args):
        try:
            ver = importlib_metadata.version("bijux-cli")
        except importlib_metadata.PackageNotFoundError:
            ver = "unknown"
        print(json.dumps({"version": ver}))
        return 0

    container = DIContainer.current()
    register_default_services(
        container, debug=False, output_format=OutputFormat.JSON, quiet=False
    )

    Engine()
    app = build_app()

    if any(a in ("-h", "--help") for a in args):
        print(get_usage_for_args(args, app))
        return 0

    missing_format_msg = check_missing_format_argument(args)
    if missing_format_msg:
        print_json_error(missing_format_msg, 2, quiet)
        return 2

    command_line = args
    start = time.time()
    exit_code = 0

    try:
        result = app(args=command_line, standalone_mode=False)
        exit_code = int(result) if isinstance(result, int) else 0
    except typer.Exit as exc:
        exit_code = exc.exit_code
    except NoSuchOption as exc:
        print_json_error(f"No such option: {exc.option_name}", 2, quiet)
        exit_code = 2
    except UsageError as exc:
        print_json_error(str(exc), 2, quiet)
        exit_code = 2
    except CommandError as exc:
        print_json_error(str(exc), 1, quiet)
        exit_code = 1
    except KeyboardInterrupt:
        print_json_error("Aborted by user", 130, quiet)
        exit_code = 130
    except Exception as exc:
        print_json_error(f"Unexpected error: {exc}", 1, quiet)
        exit_code = 1

    if should_record_command_history(command_line):
        try:
            history_service = container.resolve(History)
            history_service.add(
                command=" ".join(command_line),
                params=command_line[1:],
                success=(exit_code == 0),
                return_code=exit_code,
                duration_ms=int((time.time() - start) * 1000),
            )
        except Exception as exc:
            print(f"[error] Could not record command history: {exc}", file=sys.stderr)
            exit_code = 1

    return exit_code


if __name__ == "__main__":
    sys.exit(main())  # pragma: no cover
