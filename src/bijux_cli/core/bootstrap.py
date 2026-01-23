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
        all top-level exceptions (including `Typer` errors, custom `UserInputError`
        exceptions, and `KeyboardInterrupt`), and translates them into
        structured error messages and standardized exit codes.
    * **History Recording:** Persists the command to the history service after
        execution.
"""

from __future__ import annotations

import contextlib
from contextlib import suppress
import importlib.metadata as importlib_metadata
import json
import logging
import os
import sys
import time

from click.exceptions import NoSuchOption, UsageError
import structlog
import typer

from bijux_cli.cli.color import set_color_mode
from bijux_cli.cli.flags import parse_global_flags
from bijux_cli.cli.root import build_app
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import ColorMode, ErrorType, ExitCode, LogLevel, OutputFormat
from bijux_cli.core.errors import UserInputError
from bijux_cli.core.exit_policy import resolve_exit_behavior
from bijux_cli.core.precedence import (
    EffectiveConfig,
    ExecutionPolicy,
    FlagLayer,
    Flags,
    resolve_effective_config,
    resolve_execution_policy,
    validate_cli_flags,
)
from bijux_cli.plugins.services import register_plugin_services
from bijux_cli.services import register_default_services
from bijux_cli.services.history import History
from bijux_cli.services.logging.contracts import LoggingConfig


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


def print_json_error(
    msg: str,
    error_type: ErrorType = ErrorType.USAGE,
    quiet: bool = False,
    fmt: OutputFormat = OutputFormat.JSON,
) -> ExitCode:
    """Prints a structured JSON error message.

    The message is printed to stdout for usage errors (code 2) and stderr for
    all other errors, unless quiet mode is enabled.

    Args:
        msg (str): The error message.
        error_type (ErrorType): The error category for exit mapping.
        quiet (bool): If True, suppresses all output.
        fmt (OutputFormat): The output format to record in exit mapping.
    """
    behavior = resolve_exit_behavior(error_type, quiet=quiet, fmt=fmt)
    if behavior.stream is not None:
        stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
        print(
            json.dumps({"error": msg, "code": int(behavior.code)}),
            file=stream,
        )
    return behavior.code


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


def setup_structlog(log_level: str | None = None) -> None:
    """Configures `structlog` for the application.

    Args:
        log_level (str | None): Optional explicit log level override.
    """
    if log_level:
        level = getattr(logging, log_level.upper(), logging.CRITICAL)
    else:
        level = logging.CRITICAL
    logging.basicConfig(level=level, stream=sys.stderr, format="%(message)s")

    use_console = (log_level == LogLevel.DEBUG) or os.environ.get(
        "BIJUXCLI_TEST_MODE"
    ) == "1"
    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.stdlib.add_log_level,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.processors.UnicodeDecoder(),
            structlog.dev.ConsoleRenderer()
            if use_console
            else structlog.processors.JSONRenderer(),
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

    parsed = parse_global_flags(args)
    for err in validate_cli_flags(parsed):
        msg = err["message"]
        failure = err["failure"]
        if failure == "missing_argument" and "format" in msg.lower():
            continue
        return print_json_error(
            msg, ErrorType.USAGE, bool(parsed.flags.quiet), OutputFormat.JSON
        )
    env_log = os.environ.get("BIJUXCLI_LOG_LEVEL")
    env_color = os.environ.get("BIJUXCLI_COLOR")
    resolved = resolve_effective_config(
        cli=parsed.flags,
        env=FlagLayer(
            log_level=LogLevel(env_log) if env_log else None,
            color=ColorMode(env_color) if env_color else None,
        ),
        file=FlagLayer(),
        defaults=Flags(
            quiet=False,
            log_level=LogLevel.INFO,
            color=ColorMode.AUTO,
            format=OutputFormat.JSON,
        ),
    )

    if resolved.flags.quiet:
        with contextlib.suppress(Exception):
            sys.stderr = open(os.devnull, "w")  # noqa: SIM115
    debug_enabled = resolved.flags.log_level == LogLevel.DEBUG
    logging_config = LoggingConfig(
        debug=debug_enabled,
        quiet=resolved.flags.quiet,
        verbose=False,
        log_level=resolved.flags.log_level,
        color=resolved.flags.color,
    )
    policy = resolve_execution_policy(resolved)
    setup_structlog(resolved.flags.log_level.value)
    set_color_mode(policy.color)

    if any(a in ("--version", "-V") for a in args):
        try:
            ver = importlib_metadata.version("bijux-cli")
        except importlib_metadata.PackageNotFoundError:
            ver = "unknown"
        print(json.dumps({"version": ver}))
        return 0

    container = DIContainer.current()
    container.register(EffectiveConfig, resolved)
    container.register(ExecutionPolicy, policy)
    register_default_services(
        container,
        logging_config=logging_config,
        output_format=OutputFormat.YAML
        if policy.output_format == OutputFormat.YAML
        else OutputFormat.JSON,
    )
    register_plugin_services(container)

    Engine()
    app = build_app()

    if parsed.help:
        print(get_usage_for_args(args, app))
        return 0

    missing_format_msg = check_missing_format_argument(args)
    if missing_format_msg:
        return print_json_error(
            missing_format_msg, ErrorType.USAGE, resolved.flags.quiet, OutputFormat.JSON
        )

    command_line = list(parsed.args)
    start = time.time()
    exit_code = 0

    try:
        result = app(args=command_line, standalone_mode=False)
        exit_code = int(result) if isinstance(result, int) else 0
    except typer.Exit as exc:
        exit_code = exc.exit_code
    except NoSuchOption as exc:
        exit_code = print_json_error(
            f"No such option: {exc.option_name}",
            ErrorType.USAGE,
            resolved.flags.quiet,
            OutputFormat.JSON,
        )
    except UsageError as exc:
        exit_code = print_json_error(
            str(exc), ErrorType.USAGE, resolved.flags.quiet, OutputFormat.JSON
        )
    except UserInputError as exc:
        exit_code = print_json_error(
            str(exc), ErrorType.USER_INPUT, resolved.flags.quiet, OutputFormat.JSON
        )
    except KeyboardInterrupt:
        exit_code = print_json_error(
            "Aborted by user",
            ErrorType.ABORTED,
            resolved.flags.quiet,
            OutputFormat.JSON,
        )
    except Exception as exc:
        exit_code = print_json_error(
            f"Unexpected error: {exc}",
            ErrorType.INTERNAL,
            resolved.flags.quiet,
            OutputFormat.JSON,
        )

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
