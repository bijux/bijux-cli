# SPDX-License-Identifier: Apache-2.0
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
from bijux_cli.cli.core.command import emit_payload, resolve_emitter, resolve_serializer
from bijux_cli.cli.core.constants import ENV_DISABLE_HISTORY, ENV_TEST_MODE
from bijux_cli.cli.root import build_app
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.errors import UserInputError
from bijux_cli.core.exit_policy import resolve_exit_behavior
from bijux_cli.core.intent import CLIIntent, build_cli_intent, split_command_args
from bijux_cli.core.precedence import (
    EffectiveConfig,
    ExecutionPolicy,
    LogPolicy,
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
    # POLICY: history recording eligibility.
    if os.environ.get(ENV_DISABLE_HISTORY) == "1":
        return False
    if not command_line:
        return False
    return command_line[0].lower() not in {"history", "help"}


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

    # IO: capture help output by redirecting stdout.
    with io.StringIO() as buf, redirect_stdout(buf):
        with suppress(SystemExit):
            app(subcmds + ["--help"], standalone_mode=False)
        return buf.getvalue()


def setup_structlog(log_level: LogLevel | None = None) -> None:
    """Configures `structlog` for the application.

    Args:
        log_level (str | None): Optional explicit log level override.
    """
    # POLICY: logging level threshold for structlog.
    level = logging.DEBUG if log_level is LogLevel.DEBUG else logging.WARNING
    logging.basicConfig(level=level, stream=sys.stderr, format="%(message)s")

    # IO: environment read for test-mode logging choice.
    use_console = (log_level is LogLevel.DEBUG) or os.environ.get(ENV_TEST_MODE) == "1"
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


def _emit_fast_payload(
    payload: object,
    *,
    fmt: OutputFormat,
    stream: str,
) -> None:
    """Serialize and emit a payload without DI initialization."""
    from dataclasses import asdict, is_dataclass
    from typing import Any, cast

    # FAST PATH: emit without DI initialization.
    if is_dataclass(payload):
        payload = asdict(cast(Any, payload))
    if fmt is OutputFormat.YAML:
        try:
            import yaml
        except ImportError:
            text = json.dumps(payload)
        else:
            text = (yaml.safe_dump(payload, sort_keys=False) or "").rstrip("\n")
    else:
        text = json.dumps(payload)
    # IO: direct stream output for fast path.
    out = sys.stdout if stream == "stdout" else sys.stderr
    print(text, file=out)


def _emit_fast_error(
    message: str,
    *,
    error_type: ErrorType,
    quiet: bool,
    fmt: OutputFormat,
    log_policy: LogPolicy,
) -> int:
    """Emit a structured error payload without DI initialization."""
    # POLICY: exit behavior resolved from error type and log policy.
    behavior = resolve_exit_behavior(
        error_type, quiet=quiet, fmt=fmt, log_policy=log_policy
    )
    code = int(behavior.code)
    if behavior.stream is None:
        return code
    payload = {"error": message, "code": code}
    _emit_fast_payload(payload, fmt=fmt, stream=behavior.stream)
    return code


def _handle_version_request(args: list[str], intent: CLIIntent) -> int | None:
    """Handle version requests without initializing DI or plugins."""
    if any(a in ("--version", "-V") for a in args):
        try:
            ver = importlib_metadata.version("bijux-cli")
        except importlib_metadata.PackageNotFoundError:
            ver = "unknown"
        print(json.dumps({"version": ver}))
        return 0

    command, sub_args = split_command_args(args)
    if command != "version":
        return None

    if "-h" in sub_args or "--help" in sub_args:
        app = build_app(load_plugins=False)
        print(get_usage_for_args(["version", "--help"], app))
        return 0

    from bijux_cli.cli.commands.version import _build_payload

    if intent.quiet:
        return 0

    if intent.log_policy.show_internal:
        print("debug: fast version path", file=sys.stderr)

    try:
        payload = _build_payload(intent.include_runtime)
    except ValueError as exc:
        return _emit_fast_error(
            str(exc),
            error_type=ErrorType.CONFIG,
            quiet=intent.quiet,
            fmt=intent.output_format,
            log_policy=intent.log_policy,
        )
    _emit_fast_payload(payload, fmt=intent.output_format, stream="stdout")
    return 0


def _handle_help_request(args: list[str], intent: CLIIntent) -> int | None:
    """Handle help requests without initializing DI or plugins."""
    if not intent.help:
        return None
    app = build_app(load_plugins=False)
    print(get_usage_for_args(args, app))
    return 0


def run_runtime(intent: CLIIntent) -> int:
    """Run the DI/runtime execution path."""
    # Phase: runtime init (logging config + DI graph).
    if intent.quiet:
        with contextlib.suppress(Exception):
            sys.stderr = open(os.devnull, "w")  # noqa: SIM115

    logging_config = LoggingConfig(
        quiet=intent.quiet,
        log_level=intent.log_level,
        color=intent.color,
    )

    container = DIContainer.current()
    container.register(CLIIntent, intent)
    container.register(EffectiveConfig, EffectiveConfig(flags=intent.flags))
    container.register(
        ExecutionPolicy,
        ExecutionPolicy(
            output_format=intent.output_format,
            color=intent.color,
            quiet=intent.quiet,
            log_level=intent.log_level,
            pretty=intent.pretty,
            include_runtime=intent.include_runtime,
        ),
    )

    register_default_services(
        container,
        logging_config=logging_config,
        output_format=intent.output_format,
    )
    register_plugin_services(container)

    Engine()
    app = build_app()
    serializer = resolve_serializer()
    emitter = resolve_emitter()

    # Phase: execution + emission.
    command_line = list(intent.args)
    start = time.time()

    def emit_error(error_type: ErrorType, message: str) -> int:
        behavior = resolve_exit_behavior(
            error_type,
            quiet=intent.quiet,
            fmt=intent.output_format,
            log_policy=intent.log_policy,
        )
        # Invariant: exit behavior is resolved once; emission just executes.
        code = int(behavior.code)
        if behavior.stream is None:
            return code
        emit_payload(
            {"error": message, "code": code},
            serializer=serializer,
            emitter=emitter,
            fmt=intent.output_format,
            pretty=intent.pretty,
            stream=behavior.stream,
        )
        return code

    try:
        result = app(args=command_line, standalone_mode=False)
        exit_code = int(result) if isinstance(result, int) else 0
    except typer.Exit as exc:
        exit_code = exc.exit_code
    except NoSuchOption as exc:
        exit_code = emit_error(ErrorType.USAGE, f"No such option: {exc.option_name}")
    except UsageError as exc:
        exit_code = emit_error(ErrorType.USAGE, str(exc))
    except UserInputError as exc:
        exit_code = emit_error(ErrorType.USER_INPUT, str(exc))
    except KeyboardInterrupt:
        exit_code = emit_error(ErrorType.ABORTED, "Aborted by user")
    except Exception as exc:
        exit_code = emit_error(ErrorType.INTERNAL, f"Unexpected error: {exc}")

    # Phase: history recording.
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
    # Phase: intent building (no side effects).
    args = sys.argv[1:]
    intent = build_cli_intent(args, env=os.environ, tty=sys.stdout.isatty())
    if intent.errors:
        err = intent.errors[0]
        return _emit_fast_error(
            err.message,
            error_type=ErrorType.USAGE,
            quiet=intent.quiet,
            fmt=intent.output_format,
            log_policy=intent.log_policy,
        )

    # Phase: explicit fast paths.
    fast_exit = _handle_version_request(args, intent)
    if fast_exit is not None:
        return fast_exit
    fast_exit = _handle_help_request(args, intent)
    if fast_exit is not None:
        return fast_exit

    # Phase: policy resolution + runtime init.
    try:
        DIContainer.set_log_policy(intent.log_policy)
        setup_structlog(intent.log_level)
        set_color_mode(intent.color)
    except Exception as exc:
        return _emit_fast_error(
            f"Startup failed: {exc}",
            error_type=ErrorType.INTERNAL,
            quiet=intent.quiet,
            fmt=intent.output_format,
            log_policy=intent.log_policy,
        )

    # Phase: execution + emission + exit.
    return run_runtime(intent)


if __name__ == "__main__":
    raise SystemExit(main())  # pragma: no cover


__all__ = [
    "DIContainer",
    "Engine",
    "main",
    "register_default_services",
    "register_plugin_services",
    "run_runtime",
    "sys",
]
