# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Docs command runtime for the Bijux CLI (IO + exit behavior)."""

from __future__ import annotations

import os
from pathlib import Path

import typer
import typer.core

from bijux_cli.cli.color import resolve_click_color
from bijux_cli.cli.commands.diagnostics.docs import (
    _build_spec_payload,
    _resolve_output_target,
    _spec_mapping,
)
from bijux_cli.cli.core.command import (
    contains_non_ascii_env,
    raise_exit_intent,
    record_history,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
    ENV_DOCS_OUT,
    ENV_TEST_IO_FAIL,
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
from bijux_cli.core.enums import (
    ErrorType,
    ExitCode,
)
from bijux_cli.core.exit_policy import ExitIntent, ExitIntentError
from bijux_cli.core.precedence import (
    EffectiveConfig,
    Flags,
    OutputConfig,
    default_execution_policy,
)
from bijux_cli.core.runtime import AsyncTyper
from bijux_cli.services.diagnostics.contracts import DocsProtocol

typer.core.rich = None  # type: ignore[attr-defined]

docs_app = AsyncTyper(
    name="docs",
    help="(-h, --help) Generate API specifications (OpenAPI-like) for Bijux CLI.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

OUT_OPTION = typer.Option(
    None,
    "--out",
    "-o",
    help="Output file path or '-' for stdout. If a directory is given, a default file name is used.",
)


def _resolve_docs_service() -> DocsProtocol:
    """Resolve the docs service from the DI container."""
    return DIContainer.current().resolve(DocsProtocol)


def _resolve_docs_config() -> tuple[EffectiveConfig, OutputConfig]:
    """Resolve effective and output config for docs handling."""
    try:
        effective = DIContainer.current().resolve(EffectiveConfig)
    except Exception:
        policy = default_execution_policy()
        effective = EffectiveConfig(
            flags=Flags(
                quiet=policy.quiet,
                log_level=policy.log_level,
                color=policy.color,
                format=policy.output_format,
            )
        )
    try:
        output = DIContainer.current().resolve(OutputConfig)
    except Exception:
        policy = default_execution_policy()
        output = OutputConfig(
            include_runtime=policy.log_policy.show_internal,
            pretty=policy.log_policy.pretty_default,
            log_level=effective.flags.log_level,
            color=effective.flags.color,
            format=effective.flags.format,
            log_policy=policy.log_policy,
        )
    return effective, output


@docs_app.callback(invoke_without_command=True)
def docs(
    ctx: typer.Context,
    out: Path | None = OUT_OPTION,
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Entrypoint for the `bijux docs` command."""
    _ = (quiet, pretty, log_level)
    command = "docs"
    effective, output = _resolve_docs_config()
    effective_include_runtime = output.include_runtime
    effective_pretty = output.pretty
    log_level_value = output.log_level
    output_format = validate_common_flags(
        fmt,
        command,
        effective.flags.quiet,
        include_runtime=effective_include_runtime,
        log_level=log_level_value,
    )

    if contains_non_ascii_env():
        raise_exit_intent(
            "Non-ASCII characters in environment variables",
            code=3,
            failure="ascii_env",
            error_type=ErrorType.ASCII,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    if ctx.args:
        stray = ctx.args[0]
        msg = (
            f"No such option: {stray}"
            if stray.startswith("-")
            else f"Too many arguments: {' '.join(ctx.args)}"
        )
        raise_exit_intent(
            msg,
            code=2,
            failure="args",
            error_type=ErrorType.USAGE,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    out_env = os.environ.get(ENV_DOCS_OUT)
    if out is None and out_env:
        out = Path(out_env)

    target, path = _resolve_output_target(out, output_format)

    try:
        spec = _build_spec_payload(effective_include_runtime)
        spec_mapping = _spec_mapping(spec)
    except ValueError as exc:
        raise_exit_intent(
            str(exc),
            code=3,
            failure="ascii",
            error_type=ErrorType.ASCII,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    docs_service = _resolve_docs_service()
    try:
        content = docs_service.render(
            spec_mapping, fmt=output_format, pretty=effective_pretty
        )
    except Exception as exc:
        raise_exit_intent(
            f"Serialization failed: {exc}",
            code=1,
            failure="serialize",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    if os.environ.get(ENV_TEST_IO_FAIL) == "1":
        raise_exit_intent(
            "Simulated I/O failure for test",
            code=1,
            failure="io_fail",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    emit_output = not effective.flags.quiet
    if target == "-":
        if emit_output:
            typer.echo(
                content,
                color=resolve_click_color(quiet=False, fmt=output_format),
                err=False,
            )
        record_history(command, 0)
        raise ExitIntentError(
            ExitIntent(
                code=ExitCode.SUCCESS,
                stream=None,
                payload=None,
                fmt=output_format,
                pretty=effective_pretty,
                show_traceback=False,
            )
        )

    if path is None:
        raise_exit_intent(
            "Internal error: expected non-null output path",
            code=1,
            failure="internal",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    parent = path.parent
    if not parent.exists():
        raise_exit_intent(
            f"Output directory does not exist: {parent}",
            code=2,
            failure="output_dir",
            error_type=ErrorType.USER_INPUT,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    try:
        docs_service.write(
            spec_mapping,
            fmt=output_format,
            name=str(path),
            pretty=effective_pretty,
        )
    except Exception as exc:
        raise_exit_intent(
            f"Failed to write spec: {exc}",
            code=2,
            failure="write",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=output_format,
            quiet=effective.flags.quiet,
            include_runtime=effective_include_runtime,
            log_level=log_level_value,
        )

    record_history(command, 0)
    intent_payload = {"status": "written", "file": str(path)} if emit_output else None
    stream = "stdout" if emit_output else None
    raise ExitIntentError(
        ExitIntent(
            code=ExitCode.SUCCESS,
            stream=stream,
            payload=intent_payload,
            fmt=output_format,
            pretty=effective_pretty,
            show_traceback=False,
        )
    )


__all__ = ["docs_app", "docs"]
