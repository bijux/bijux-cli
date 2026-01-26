# SPDX-License-Identifier: Apache-2.0
# Copyright © 2025 Bijan Mousavi

"""Implements the `plugins scaffold` subcommand for the Bijux CLI.

This module contains the logic for creating a new plugin project from a
`cookiecutter` template. It validates the proposed plugin name, handles the
destination directory setup (including forcing overwrites), and invokes
`cookiecutter` to generate the project structure.

Output Contract:
    * Success: `{"status": "created", "plugin": str, "dir": str}`
    * Error:   `{"error": "...", "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal error occurred (e.g., cookiecutter not installed, invalid
      template, name conflict, filesystem error).
    * `2`: An invalid flag was provided (e.g., bad format).
    * `3`: An ASCII or encoding error was detected in the environment.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
import keyword
from pathlib import Path
import shutil
import unicodedata

import typer

from bijux_cli.cli.core.command import (
    new_run_command,
    raise_exit_intent,
    validate_common_flags,
)
from bijux_cli.cli.core.constants import (
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
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.plugins.catalog import PLUGIN_NAME_RE


@dataclass(frozen=True)
class ScaffoldIntent:
    """Resolved intent for plugin scaffolding."""

    name: str
    template: str
    target: Path
    force: bool
    quiet: bool
    include_runtime: bool
    log_level: LogLevel
    fmt: OutputFormat


def _build_scaffold_intent(
    *,
    name: str,
    output_dir: str,
    template: str | None,
    force: bool,
    command: str,
    fmt: OutputFormat,
    quiet: bool,
    include_runtime: bool,
    log_level: LogLevel,
) -> ScaffoldIntent:
    """Validate inputs and build a scaffold intent."""
    if name in keyword.kwlist:
        raise_exit_intent(
            f"Invalid plugin name: '{name}' is a reserved Python keyword.",
            code=1,
            failure="reserved_keyword",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )

    if not PLUGIN_NAME_RE.fullmatch(name) or not name.isascii():
        raise_exit_intent(
            "Invalid plugin name: only ASCII letters, digits, dash and underscore are allowed.",
            code=1,
            failure="invalid_name",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )

    if not template:
        raise_exit_intent(
            "No plugin template found. Please specify --template (path or URL).",
            code=1,
            failure="no_template",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )
    if template is None:
        raise RuntimeError("Template must be provided")

    slug = unicodedata.normalize("NFC", name)
    parent = Path(output_dir).expanduser().resolve()
    target = parent / slug

    if not parent.exists():
        try:
            parent.mkdir(parents=True, exist_ok=True)
        except Exception as exc:
            raise_exit_intent(
                f"Failed to create output directory '{parent}': {exc}",
                code=1,
                failure="create_dir_failed",
                command=command,
                fmt=fmt,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level,
            )
    elif not parent.is_dir():
        raise_exit_intent(
            f"Output directory '{parent}' is not a directory.",
            code=1,
            failure="not_dir",
            command=command,
            fmt=fmt,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
        )

    normalized = name.lower()
    for existing in parent.iterdir():
        if (
            (existing.is_dir() or existing.is_symlink())
            and existing.name.lower() == normalized
            and existing.resolve() != target.resolve()
        ):
            raise_exit_intent(
                f"Plugin name '{name}' conflicts with existing directory '{existing.name}'. "
                "Plugin names must be unique (case-insensitive).",
                code=1,
                failure="name_conflict",
                command=command,
                fmt=fmt,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level,
            )

    if target.exists() or target.is_symlink():
        if not force:
            raise_exit_intent(
                f"Directory '{target}' is not empty – use --force to overwrite.",
                code=1,
                failure="dir_not_empty",
                command=command,
                fmt=fmt,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level,
            )
        try:
            if target.is_symlink():
                target.unlink()
            elif target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
        except Exception as exc:
            raise_exit_intent(
                f"Failed to remove existing '{target}': {exc}",
                code=1,
                failure="remove_failed",
                command=command,
                fmt=fmt,
                quiet=quiet,
                include_runtime=include_runtime,
                log_level=log_level,
            )

    return ScaffoldIntent(
        name=name,
        template=template,
        target=target,
        force=force,
        quiet=quiet,
        include_runtime=include_runtime,
        log_level=log_level,
        fmt=fmt,
    )


def _scaffold_project(intent: ScaffoldIntent) -> dict[str, str]:
    """Run cookiecutter and validate the output."""
    try:
        from cookiecutter.main import cookiecutter

        cookiecutter(
            intent.template,
            no_input=True,
            output_dir=str(intent.target.parent),
            extra_context={
                "project_name": intent.name,
                "project_slug": intent.target.name,
            },
        )
        if not intent.target.is_dir():
            raise RuntimeError("Template copy failed")
    except ModuleNotFoundError:
        raise_exit_intent(
            "cookiecutter is required but not installed.",
            code=1,
            failure="cookiecutter_missing",
            command="plugins scaffold",
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )
    except Exception as exc:
        msg = f"Scaffold failed: {exc} (template not found or invalid)"
        raise_exit_intent(
            msg,
            code=1,
            failure="scaffold_failed",
            command="plugins scaffold",
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )

    plugin_json = intent.target / "plugin.json"
    if not plugin_json.is_file():
        raise_exit_intent(
            f"Scaffold failed: plugin.json not found in '{intent.target}'.",
            code=1,
            failure="plugin_json_missing",
            command="plugins scaffold",
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )
    try:
        meta = json.loads(plugin_json.read_text("utf-8"))
        if not (
            isinstance(meta, dict)
            and meta.get("name")
            and (meta.get("desc") or meta.get("description"))
            and meta.get("schema_version")
            and meta.get("bijux_cli_version")
        ):
            raise ValueError("Missing required fields")
    except Exception as exc:
        raise_exit_intent(
            f"Scaffold failed: plugin.json invalid: {exc}",
            code=1,
            failure="plugin_json_invalid",
            command="plugins scaffold",
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )

    return {"status": "created", "plugin": intent.name, "dir": str(intent.target)}


def scaffold_plugin(
    name: str = typer.Argument(..., help="Plugin name"),
    output_dir: str = typer.Option(".", "--output-dir", "-o"),
    template: str | None = typer.Option(
        None,
        "--template",
        "-t",
        help="Path or URL to a cookiecutter template (required)",
    ),
    force: bool = typer.Option(False, "--force", "-F"),
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
) -> None:
    """Creates a new plugin project from a cookiecutter template.

    This function orchestrates the scaffolding process. It performs numerous
    validations on the plugin name and output directory, handles existing
    directories with the `--force` flag, invokes the `cookiecutter` library
    to generate the project, and validates the resulting plugin metadata.

    Args:
        name (str): The name for the new plugin (e.g., 'my-plugin').
        output_dir (str): The directory where the new plugin project will be
            created.
        template (str | None): The path or URL to the `cookiecutter` template.
        force (bool): If True, overwrites the output directory if it exists.
        quiet (bool): If True, suppresses all output except for errors.
        fmt (str): The output format for confirmation or error messages.
        pretty (bool): If True, pretty-prints the output.
        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins scaffold"

    policy = current_execution_policy()
    quiet = policy.quiet
    include_runtime = policy.include_runtime
    log_level_value = policy.log_level
    pretty = policy.pretty
    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )

    intent = _build_scaffold_intent(
        name=name,
        output_dir=output_dir,
        template=template,
        force=force,
        command=command,
        fmt=fmt_lower,
        quiet=quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )
    payload = _scaffold_project(intent)

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level_value,
    )
