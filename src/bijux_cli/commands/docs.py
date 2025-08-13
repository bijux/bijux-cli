# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Docs command for the Bijux CLI.

Generates a machine-readable specification of the entire CLI, outputting it as
JSON or YAML. This command is designed for automation, enabling integration
with external documentation tools or APIs. It supports outputting to stdout or
a file and ensures all text is ASCII-safe.

Output Contract:
    * Success (file):   `{"status": "written", "file": "<path>"}`
    * Success (stdout): The raw specification string is printed directly.
    * Spec fields:      `{"version": str, "commands": list, ...}`
    * Verbose:          Adds `{"python": str, "platform": str}` to the spec.
    * Error:            `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: Fatal or internal error.
    * `2`: CLI argument, flag, or format error.
    * `3`: ASCII or encoding error.
"""

from __future__ import annotations

from collections.abc import Mapping
import os
from pathlib import Path
import platform

import typer
import typer.core

from bijux_cli.__version__ import __version__
from bijux_cli.commands.utilities import (
    contains_non_ascii_env,
    emit_and_exit,
    emit_error_and_exit,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.enums import OutputFormat

typer.core.rich = None  # type: ignore[attr-defined,assignment]

docs_app = typer.Typer(  # pytype: skip-file
    name="docs",
    help="(-h, --help) Generate API specifications (OpenAPI-like) for Bijux CLI.",
    rich_markup_mode=None,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=False,
)

CLI_VERSION = __version__


def _default_output_path(base: Path, fmt: str) -> Path:
    """Computes the default output file path for a CLI spec.

    Args:
        base (Path): The output directory path.
        fmt (str): The output format extension, either "json" or "yaml".

    Returns:
        Path: The fully resolved path to the output specification file.
    """
    return base / f"spec.{fmt}"


def _resolve_output_target(out: Path | None, fmt: str) -> tuple[str, Path | None]:
    """Resolves the output target and file path for the CLI spec.

    Determines if the output should go to stdout or a file, resolving the
    final path if a directory is provided.

    Args:
        out (Path | None): The user-provided output path, which can be a file,
            a directory, or '-' for stdout.
        fmt (str): The output format extension ("json" or "yaml").

    Returns:
        tuple[str, Path | None]: A tuple containing the target and path. The
            target is a string ("-" for stdout or a file path), and the path
            is the resolved `Path` object or `None` for stdout.
    """
    if out is None:
        path = _default_output_path(Path.cwd(), fmt)
        return str(path), path
    if str(out) == "-":
        return "-", None
    if out.is_dir():
        path = _default_output_path(out, fmt)
        return str(path), path
    return str(out), out


def _build_spec_payload(include_runtime: bool) -> Mapping[str, object]:
    """Builds the CLI specification payload.

    Args:
        include_runtime (bool): If True, includes Python and platform metadata
            in the specification.

    Returns:
        Mapping[str, object]: A dictionary containing the CLI version, a list
            of registered commands, and optional runtime details.

    Raises:
        ValueError: If the version string or platform metadata contains
            non-ASCII characters.
    """
    from bijux_cli.commands import list_registered_command_names
    from bijux_cli.commands.utilities import ascii_safe

    version_str = ascii_safe(CLI_VERSION, "version")
    payload: dict[str, object] = {
        "version": version_str,
        "commands": list_registered_command_names(),
    }
    if include_runtime:
        payload["python"] = ascii_safe(platform.python_version(), "python_version")
        payload["platform"] = ascii_safe(platform.platform(), "platform")
    return payload


OUT_OPTION = typer.Option(
    None,
    "--out",
    "-o",
    help="Output file path or '-' for stdout. If a directory is given, a default file name is used.",
)


@docs_app.callback(invoke_without_command=True)
def docs(
    ctx: typer.Context,
    out: Path | None = OUT_OPTION,
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Defines the entrypoint and logic for the `bijux docs` command.

    This function orchestrates the entire specification generation process. It
    validates CLI flags, checks for ASCII-safe environment variables, resolves
    the output destination, builds the specification payload, and writes the
    result to a file or stdout. All errors are handled and emitted in a
    structured format before exiting with a specific code.

    Args:
        ctx (typer.Context): The Typer context, used for managing command state.
        out (Path | None): The output destination: a file path, a directory, or
            '-' to signify stdout.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python and platform metadata in the spec.
        fmt (str): The output format, either "json" or "yaml". Defaults to "json".
        pretty (bool): If True, pretty-prints the output for human readability.
        debug (bool): If True, enables debug diagnostics, implying `verbose`
            and `pretty`.

    Returns:
        None:

    Raises:
        SystemExit: Exits the application with a contract-compliant status code
            and payload upon any error, including argument validation, ASCII
            violations, serialization failures, or I/O issues.
    """
    from bijux_cli.commands.utilities import normalize_format
    from bijux_cli.infra.serializer import OrjsonSerializer, PyYAMLSerializer
    from bijux_cli.infra.telemetry import NullTelemetry

    command = "docs"
    effective_include_runtime = (verbose or debug) and not quiet
    effective_pretty = True if (debug and not quiet) else pretty

    fmt_lower = normalize_format(fmt)

    if ctx.args:
        stray = ctx.args[0]
        msg = (
            f"No such option: {stray}"
            if stray.startswith("-")
            else f"Too many arguments: {' '.join(ctx.args)}"
        )
        emit_error_and_exit(
            msg,
            code=2,
            failure="args",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=effective_include_runtime,
    )

    if contains_non_ascii_env():
        emit_error_and_exit(
            "Non-ASCII characters in environment variables",
            code=3,
            failure="ascii_env",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    out_env = os.environ.get("BIJUXCLI_DOCS_OUT")
    if out is None and out_env:
        out = Path(out_env)

    target, path = _resolve_output_target(out, fmt_lower)

    try:
        spec = _build_spec_payload(effective_include_runtime)
    except ValueError as exc:
        emit_error_and_exit(
            str(exc),
            code=3,
            failure="ascii",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    output_format = OutputFormat.YAML if fmt_lower == "yaml" else OutputFormat.JSON
    serializer = (
        PyYAMLSerializer(NullTelemetry())
        if output_format is OutputFormat.YAML
        else OrjsonSerializer(NullTelemetry())
    )
    try:
        content = serializer.dumps(spec, fmt=output_format, pretty=effective_pretty)
    except Exception as exc:
        emit_error_and_exit(
            f"Serialization failed: {exc}",
            code=1,
            failure="serialize",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    if os.environ.get("BIJUXCLI_TEST_IO_FAIL") == "1":
        emit_error_and_exit(
            "Simulated I/O failure for test",
            code=1,
            failure="io_fail",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    if target == "-":
        if not quiet:
            typer.echo(content)
        raise typer.Exit(0)

    if path is None:
        emit_error_and_exit(
            "Internal error: expected non-null output path",
            code=1,
            failure="internal",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    parent = path.parent
    if not parent.exists():
        emit_error_and_exit(
            f"Output directory does not exist: {parent}",
            code=2,
            failure="output_dir",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    try:
        path.write_text(content, encoding="utf-8")
    except Exception as exc:
        emit_error_and_exit(
            f"Failed to write spec: {exc}",
            code=2,
            failure="write",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=effective_include_runtime,
            debug=debug,
        )

    emit_and_exit(
        {"status": "written", "file": str(path)},
        output_format,
        effective_pretty,
        verbose,
        debug,
        quiet,
        command,
    )
