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

from bijux_cli.cli.color import resolve_color_mode, set_color_mode
from bijux_cli.cli.core.constants import (
    ENV_COLOR,
    ENV_DISABLE_HISTORY,
    ENV_LOG_LEVEL,
    ENV_NO_COLOR,
    ENV_TEST_MODE,
)
from bijux_cli.cli.root import build_app, parse_global_config
from bijux_cli.core.di import DIContainer
from bijux_cli.core.engine import Engine
from bijux_cli.core.enums import ColorMode, ErrorType, LogLevel, OutputFormat
from bijux_cli.core.errors import UserInputError
from bijux_cli.core.exit_policy import resolve_exit_behavior
from bijux_cli.core.precedence import (
    EffectiveConfig,
    ExecutionPolicy,
    FlagLayer,
    Flags,
    GlobalCLIConfig,
    LogPolicy,
    default_execution_policy,
    resolve_effective_config,
    resolve_execution_policy,
    resolve_log_policy,
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

    with io.StringIO() as buf, redirect_stdout(buf):
        with suppress(SystemExit):
            app(subcmds + ["--help"], standalone_mode=False)
        return buf.getvalue()


def setup_structlog(log_level: LogLevel | None = None) -> None:
    """Configures `structlog` for the application.

    Args:
        log_level (str | None): Optional explicit log level override.
    """
    level = logging.DEBUG if log_level is LogLevel.DEBUG else logging.WARNING
    logging.basicConfig(level=level, stream=sys.stderr, format="%(message)s")

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
    behavior = resolve_exit_behavior(
        error_type, quiet=quiet, fmt=fmt, log_policy=log_policy
    )
    code = int(behavior.code)
    if behavior.stream is None:
        return code
    payload = {"error": message, "code": code}
    _emit_fast_payload(payload, fmt=fmt, stream=behavior.stream)
    return code


def _resolve_effective_flags(parsed: GlobalCLIConfig) -> EffectiveConfig:
    """Resolve effective flags from CLI and environment sources."""
    env_log = os.environ.get(ENV_LOG_LEVEL)
    env_color = os.environ.get(ENV_COLOR)
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

    color_config = GlobalCLIConfig(
        help=parsed.help,
        flags=FlagLayer(color=resolved.flags.color),
        args=parsed.args,
        errors=parsed.errors,
    )
    resolved_color = resolve_color_mode(
        color_config,
        sys.stdout.isatty(),
        no_color=os.environ.get(ENV_NO_COLOR) == "1",
    )
    if resolved_color != resolved.flags.color:
        return EffectiveConfig(
            flags=Flags(
                quiet=resolved.flags.quiet,
                log_level=resolved.flags.log_level,
                color=resolved_color,
                format=resolved.flags.format,
            )
        )
    return resolved


def _split_command_args(args: list[str]) -> tuple[str | None, list[str]]:
    """Return the first command token and remaining args."""
    from bijux_cli.cli.core.constants import (
        OPT_COLOR,
        OPT_FORMAT,
        OPT_HELP,
        OPT_LOG_LEVEL,
        OPT_QUIET,
        OPT_VERBOSE,
        PRETTY_FLAGS,
    )

    flags_with_values = {*OPT_FORMAT, *OPT_LOG_LEVEL, *OPT_COLOR}
    flags_no_values = {*OPT_HELP, *OPT_QUIET, *OPT_VERBOSE, *PRETTY_FLAGS}
    i = 0
    while i < len(args):
        arg = args[i]
        if arg in flags_with_values:
            i += 2
            continue
        if arg in flags_no_values or arg.startswith("-"):
            i += 1
            continue
        return arg, args[i + 1 :]
    return None, []


def _maybe_fast_help(
    args: list[str],
    parsed: GlobalCLIConfig,
    policy: ExecutionPolicy,
) -> int | None:
    """Handle --help without initializing DI or plugins."""
    if not parsed.help:
        return None
    app = build_app(load_plugins=False)
    print(get_usage_for_args(args, app))
    return 0


def _maybe_fast_version(
    args: list[str],
    policy: ExecutionPolicy,
) -> int | None:
    """Handle `bijux version` without initializing DI or plugins."""
    command, sub_args = _split_command_args(args)
    if command != "version":
        return None

    if "-h" in sub_args or "--help" in sub_args:
        app = build_app(load_plugins=False)
        print(get_usage_for_args(["version", "--help"], app))
        return 0

    from bijux_cli.cli.commands.version import _build_payload
    from bijux_cli.cli.core.constants import (
        OPT_FORMAT,
        OPT_LOG_LEVEL,
        OPT_QUIET,
        OPT_VERBOSE,
    )
    from bijux_cli.cli.core.validation import normalize_format

    quiet = policy.quiet
    verbose = False
    fmt_value: str | OutputFormat = OutputFormat.JSON
    log_level = policy.log_level

    i = 0
    while i < len(sub_args):
        arg = sub_args[i]
        if arg in OPT_QUIET:
            quiet = True
            i += 1
            continue
        if arg in OPT_VERBOSE:
            verbose = True
            i += 1
            continue
        if arg in OPT_FORMAT:
            try:
                fmt_value = sub_args[i + 1]
            except IndexError:
                return _emit_fast_error(
                    "Missing value for --format.",
                    error_type=ErrorType.USAGE,
                    quiet=quiet,
                    fmt=OutputFormat.JSON,
                    log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
                )
            i += 2
            continue
        if arg in OPT_LOG_LEVEL:
            try:
                log_level = LogLevel(sub_args[i + 1])
            except IndexError:
                return _emit_fast_error(
                    "Missing value for --log-level.",
                    error_type=ErrorType.USAGE,
                    quiet=quiet,
                    fmt=OutputFormat.JSON,
                    log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
                )
            except ValueError:
                return _emit_fast_error(
                    "Invalid log level.",
                    error_type=ErrorType.USAGE,
                    quiet=quiet,
                    fmt=OutputFormat.JSON,
                    log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
                )
            i += 2
            continue
        if arg == "--pretty":
            i += 1
            continue
        if arg == "--no-pretty":
            i += 1
            continue
        return _emit_fast_error(
            f"Unsupported option: {arg}",
            error_type=ErrorType.USAGE,
            quiet=quiet,
            fmt=OutputFormat.JSON,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )

    fmt_normalized = normalize_format(fmt_value)
    if fmt_normalized is None:
        return _emit_fast_error(
            f"Unsupported format: {fmt_value}",
            error_type=ErrorType.USAGE,
            quiet=quiet,
            fmt=OutputFormat.JSON,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )

    if resolve_log_policy(log_level).show_internal:
        print("debug: fast version path", file=sys.stderr)

    if quiet:
        return 0

    include_runtime = verbose or policy.include_runtime
    try:
        payload = _build_payload(include_runtime)
    except ValueError as exc:
        return _emit_fast_error(
            str(exc),
            error_type=ErrorType.CONFIG,
            quiet=quiet,
            fmt=fmt_normalized,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )
    _emit_fast_payload(payload, fmt=fmt_normalized, stream="stdout")
    return 0


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
    args = sys.argv[1:]

    parsed = parse_global_config(args)
    errors = validate_cli_flags(parsed)
    if errors:
        policy = default_execution_policy()
        for err in errors:
            behavior = resolve_exit_behavior(
                ErrorType.USAGE,
                quiet=bool(parsed.flags.quiet),
                fmt=parsed.flags.format or OutputFormat.JSON,
                log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
            )
            if behavior.stream is not None:
                stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
                print(
                    json.dumps({"error": err.message, "code": int(behavior.code)}),
                    file=stream,
                )
            return int(behavior.code)

    resolved = _resolve_effective_flags(parsed)
    policy = resolve_execution_policy(resolved)
    DIContainer.set_log_policy(policy.log_policy or resolve_log_policy(LogLevel.INFO))

    setup_structlog(resolved.flags.log_level)
    set_color_mode(policy.color)

    if any(a in ("--version", "-V") for a in args):
        try:
            ver = importlib_metadata.version("bijux-cli")
        except importlib_metadata.PackageNotFoundError:
            ver = "unknown"
        print(json.dumps({"version": ver}))
        return 0

    fast_help = _maybe_fast_help(args, parsed, policy)
    if fast_help is not None:
        return fast_help

    fast_version = _maybe_fast_version(args, policy)
    if fast_version is not None:
        return fast_version

    if resolved.flags.quiet:
        with contextlib.suppress(Exception):
            sys.stderr = open(os.devnull, "w")  # noqa: SIM115

    logging_config = LoggingConfig(
        debug=policy.log_policy.show_internal if policy.log_policy else False,
        quiet=resolved.flags.quiet,
        verbose=False,
        log_level=resolved.flags.log_level,
        color=resolved.flags.color,
    )

    container = DIContainer.current()
    container.register(GlobalCLIConfig, parsed)
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

    command_line = list(parsed.args)
    start = time.time()
    exit_code = 0

    try:
        result = app(args=command_line, standalone_mode=False)
        exit_code = int(result) if isinstance(result, int) else 0
    except typer.Exit as exc:
        exit_code = exc.exit_code
    except NoSuchOption as exc:
        behavior = resolve_exit_behavior(
            ErrorType.USAGE,
            quiet=resolved.flags.quiet,
            fmt=resolved.flags.format,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )
        if behavior.stream is not None:
            stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
            print(
                json.dumps(
                    {
                        "error": f"No such option: {exc.option_name}",
                        "code": int(behavior.code),
                    }
                ),
                file=stream,
            )
        exit_code = int(behavior.code)
    except UsageError as exc:
        behavior = resolve_exit_behavior(
            ErrorType.USAGE,
            quiet=resolved.flags.quiet,
            fmt=resolved.flags.format,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )
        if behavior.stream is not None:
            stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
            print(
                json.dumps({"error": str(exc), "code": int(behavior.code)}),
                file=stream,
            )
        exit_code = int(behavior.code)
    except UserInputError as exc:
        behavior = resolve_exit_behavior(
            ErrorType.USER_INPUT,
            quiet=resolved.flags.quiet,
            fmt=resolved.flags.format,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )
        if behavior.stream is not None:
            stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
            print(
                json.dumps({"error": str(exc), "code": int(behavior.code)}),
                file=stream,
            )
        exit_code = int(behavior.code)
    except KeyboardInterrupt:
        behavior = resolve_exit_behavior(
            ErrorType.ABORTED,
            quiet=resolved.flags.quiet,
            fmt=resolved.flags.format,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )
        if behavior.stream is not None:
            stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
            print(
                json.dumps({"error": "Aborted by user", "code": int(behavior.code)}),
                file=stream,
            )
        exit_code = int(behavior.code)
    except Exception as exc:
        behavior = resolve_exit_behavior(
            ErrorType.INTERNAL,
            quiet=resolved.flags.quiet,
            fmt=resolved.flags.format,
            log_policy=policy.log_policy or resolve_log_policy(LogLevel.INFO),
        )
        if behavior.stream is not None:
            stream = sys.stdout if behavior.stream == "stdout" else sys.stderr
            print(
                json.dumps(
                    {"error": f"Unexpected error: {exc}", "code": int(behavior.code)}
                ),
                file=stream,
            )
        exit_code = int(behavior.code)

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
