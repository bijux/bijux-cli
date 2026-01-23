# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Audit command for the Bijux CLI.

Audits the current environment and configuration, emitting machine-readable structured
output in JSON or YAML. Supports dry-run simulation and writing results to a file.
Handles ASCII hygiene and structured error contracts. Output is automation-safe and
suitable for scripting or monitoring.

Output Contract:
    * Success: `{"status": "completed"}`
    * Dry-run: `{"status": "dry-run"}`
    * Written: `{"status": "written", "file": "<path>"}`
    * Verbose: `{"python": str, "platform": str}`
    * Error:   `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success, dry-run, or write success.
    * `1`: Unexpected/internal error.
    * `2`: CLI argument/flag/format or output-path error.
    * `3`: ASCII/encoding error.
"""

from __future__ import annotations

import os
from pathlib import Path
import platform

import typer

from bijux_cli.cli.commands.payloads import AuditPayload
from bijux_cli.cli.core.constants import (
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
from bijux_cli.cli.core.validation import (
    ascii_safe,
    contains_non_ascii_env,
    normalize_format,
    validate_common_flags,
    validate_env_file_if_present,
)
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.runtime import AsyncTyper
from bijux_cli.infra.contracts import Emitter

typer.core.rich = None  # type: ignore[attr-defined,assignment]

audit_app = AsyncTyper(
    name="audit",
    help="Audit the current environment for configuration and state issues.",
    rich_markup_mode=None,
    context_settings={
        "help_option_names": ["-h", "--help"],
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
    no_args_is_help=False,
)


OUTPUT_OPTION = typer.Option(
    None, "--output", "-o", help="Write output to file instead of stdout."
)
DRY_RUN_OPTION = typer.Option(
    False, "--dry-run", help="Simulate audit without making changes."
)


def _build_payload(include_runtime: bool, dry_run: bool) -> AuditPayload:
    """Builds the structured result payload for the audit command.

    Args:
        include_runtime (bool): If True, runtime metadata (Python version,
            platform) is included in the payload.
        dry_run (bool): If True, indicates the audit is a simulation, which
            sets the status field in the payload to "dry-run".

    Returns:
        Mapping[str, object]: A dictionary containing the structured audit results.
    """
    payload = AuditPayload(status="dry-run" if dry_run else "completed")
    if include_runtime:
        return AuditPayload(
            status=payload.status,
            python=ascii_safe(platform.python_version(), "python_version"),
            platform=ascii_safe(platform.platform(), "platform"),
        )
    return payload


def _write_output_file(
    output_path: Path,
    payload: object,
    emitter: Emitter,
    fmt: OutputFormat,
    pretty: bool,
    debug: bool,
    dry_run: bool,
) -> None:
    """Writes the audit payload to a specified file.

    This function serializes the payload to JSON or YAML and writes it to the
    given file path. It will fail if the parent directory does not exist.

    Args:
        output_path (Path): The file path where the payload will be written.
        payload (Mapping[str, object]): The data to serialize and write.
        emitter (Emitter): The service responsible for serialization and
            output.
        fmt (OutputFormat): The desired output format (JSON or YAML).
        pretty (bool): If True, the output is formatted for human readability.
        debug (bool): If True, enables debug-level logging during emission.
        dry_run (bool): If True, logs a message indicating a dry run.

    Returns:
        None:

    Raises:
        OSError: If the parent directory of `output_path` does not exist.
    """
    if not output_path.parent.exists():
        raise OSError(f"Output directory does not exist: {output_path.parent}")

    emitter.emit(
        payload,
        fmt=fmt,
        pretty=pretty,
        level=LogLevel.INFO,
        message="Audit dry-run completed" if dry_run else "Audit completed",
        output=str(output_path),
        emit_output=True,
        emit_diagnostics=debug,
    )


@audit_app.callback(invoke_without_command=True)
def audit(
    ctx: typer.Context,
    dry_run: bool = DRY_RUN_OPTION,
    output: Path | None = OUTPUT_OPTION,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    verbose: bool = typer.Option(False, *OPT_VERBOSE, help=HELP_VERBOSE),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Defines the entrypoint and logic for the `bijux audit` command.

    This function orchestrates the entire audit process. It validates all CLI
    flags and arguments, performs environment checks (e.g., for non-ASCII
    characters), builds the appropriate result payload, and emits it to
    stdout or a file in the specified format. All errors are handled and
    emitted in a structured format before exiting with a specific code.

    Args:
        ctx (typer.Context): The Typer context, used to manage command state
            and detect stray arguments.
        dry_run (bool): If True, simulates the audit and reports a "dry-run"
            status without performing actions.
        output (Path | None): If a path is provided, writes the audit result
            to the specified file instead of stdout.
        quiet (bool): If True, suppresses all output except for errors. The
            exit code is the primary indicator of the outcome.
        verbose (bool): If True, includes Python and platform details in the
            output payload.
        fmt (str): The output format, either "json" or "yaml". Defaults to "json".
        pretty (bool): If True, pretty-prints the output for human readability.
            This is overridden by `debug`.
        debug (bool): If True, enables debug diagnostics, which implies `verbose`
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Exits with a status code and structured error payload upon
            validation failures (e.g., bad arguments, ASCII errors), I/O
            issues, or unexpected exceptions. The exit code follows the
            contract defined in the module docstring.
    """
    if ctx.invoked_subcommand:
        return

    command = "audit"
    validate_common_flags(fmt, command, quiet)

    effective, _, fmt_lower = resolve_command_config(
        command=command,
        quiet=quiet,
        verbose=verbose,
        log_level=log_level,
        fmt=fmt,
        pretty=pretty,
    )
    include_runtime = effective.include_runtime
    effective_pretty = effective.pretty

    try:
        stray_args = [a for a in ctx.args if not a.startswith("-")]
        if stray_args:
            raise typer.BadParameter(f"No such argument: {stray_args[0]}")
        out_format = fmt_lower
        if contains_non_ascii_env():
            emit_error_and_exit(
                "Non-ASCII environment variables detected",
                code=3,
                failure="ascii_env",
                command=command,
                fmt=fmt_lower,
                quiet=effective.quiet,
                include_runtime=include_runtime,
            )
        try:
            validate_env_file_if_present(os.environ.get(ENV_CONFIG, ""))
        except ValueError as exc:
            emit_error_and_exit(
                str(exc),
                code=3,
                failure="ascii",
                command=command,
                fmt=fmt_lower,
                quiet=effective.quiet,
                include_runtime=include_runtime,
            )

    except typer.BadParameter as exc:
        error_fmt = normalize_format(fmt) or OutputFormat.JSON
        emit_error_and_exit(
            exc.message,
            code=2,
            failure="args",
            command=command,
            fmt=error_fmt,
            quiet=effective.quiet,
            include_runtime=include_runtime,
        )

    try:
        emitter = DIContainer.current().resolve(Emitter)
        payload = _build_payload(include_runtime, dry_run)

        if output is not None:
            _write_output_file(
                output_path=output,
                payload=payload,
                emitter=emitter,
                fmt=out_format,
                pretty=effective_pretty,
                debug=(effective.log_policy.show_internal),
                dry_run=dry_run,
            )
            payload = AuditPayload(status="written", file=str(output))
            if include_runtime:
                payload = AuditPayload(
                    status=payload.status,
                    file=payload.file,
                    python=ascii_safe(platform.python_version(), "python_version"),
                    platform=ascii_safe(platform.platform(), "platform"),
                )

        new_run_command(
            command_name=command,
            payload_builder=lambda _: payload,
            quiet=effective.quiet,
            verbose=include_runtime,
            fmt=fmt_lower,
            pretty=effective_pretty,
            log_level=effective.log_level,
        )

    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt_lower,
            quiet=effective.quiet,
            include_runtime=include_runtime,
        )
    except OSError as exc:
        emit_error_and_exit(
            str(exc),
            code=2,
            failure="output_file",
            command=command,
            fmt=fmt_lower,
            quiet=effective.quiet,
            include_runtime=include_runtime,
            error_type=ErrorType.USER_INPUT,
            log_policy=effective.log_policy,
        )
    except Exception as exc:
        emit_error_and_exit(
            f"An unexpected error occurred: {exc}",
            code=1,
            failure="unexpected",
            command=command,
            fmt=fmt_lower,
            quiet=effective.quiet,
            include_runtime=include_runtime,
        )
