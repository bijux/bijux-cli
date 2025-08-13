# SPDX-License-Identifier: MIT
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

import json
import keyword
from pathlib import Path
import shutil
import unicodedata

import typer

from bijux_cli.commands.plugins.utils import PLUGIN_NAME_RE
from bijux_cli.commands.utilities import (
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)


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
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
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
        verbose (bool): If True, includes runtime metadata in error payloads.
        fmt (str): The output format for confirmation or error messages.
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload, indicating success or detailing an error.
    """
    command = "plugins scaffold"

    fmt_lower = validate_common_flags(fmt, command, quiet)

    if name in keyword.kwlist:
        emit_error_and_exit(
            f"Invalid plugin name: '{name}' is a reserved Python keyword.",
            code=1,
            failure="reserved_keyword",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    if not PLUGIN_NAME_RE.fullmatch(name) or not name.isascii():
        emit_error_and_exit(
            "Invalid plugin name: only ASCII letters, digits, dash and underscore are allowed.",
            code=1,
            failure="invalid_name",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    if not template:
        emit_error_and_exit(
            "No plugin template found. Please specify --template (path or URL).",
            code=1,
            failure="no_template",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    slug = unicodedata.normalize("NFC", name)
    parent = Path(output_dir).expanduser().resolve()
    target = parent / slug

    if not parent.exists():
        try:
            parent.mkdir(parents=True, exist_ok=True)
        except Exception as exc:
            emit_error_and_exit(
                f"Failed to create output directory '{parent}': {exc}",
                code=1,
                failure="create_dir_failed",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )
    elif not parent.is_dir():
        emit_error_and_exit(
            f"Output directory '{parent}' is not a directory.",
            code=1,
            failure="not_dir",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    normalized = name.lower()
    for existing in parent.iterdir():
        if (
            (existing.is_dir() or existing.is_symlink())
            and existing.name.lower() == normalized
            and existing.resolve() != target.resolve()
        ):
            emit_error_and_exit(
                f"Plugin name '{name}' conflicts with existing directory '{existing.name}'. "
                "Plugin names must be unique (case-insensitive).",
                code=1,
                failure="name_conflict",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

    if target.exists() or target.is_symlink():
        if not force:
            emit_error_and_exit(
                f"Directory '{target}' is not empty – use --force to overwrite.",
                code=1,
                failure="dir_not_empty",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )
        try:
            if target.is_symlink():
                target.unlink()
            elif target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
        except Exception as exc:
            emit_error_and_exit(
                f"Failed to remove existing '{target}': {exc}",
                code=1,
                failure="remove_failed",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=verbose,
                debug=debug,
            )

    try:
        from cookiecutter.main import cookiecutter

        cookiecutter(
            template,
            no_input=True,
            output_dir=str(parent),
            extra_context={"project_name": name, "project_slug": slug},
        )
        if not target.is_dir():
            raise RuntimeError("Template copy failed")
    except ModuleNotFoundError:
        emit_error_and_exit(
            "cookiecutter is required but not installed.",
            code=1,
            failure="cookiecutter_missing",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )
    except Exception as exc:
        msg = f"Scaffold failed: {exc} (template not found or invalid)"
        emit_error_and_exit(
            msg,
            code=1,
            failure="scaffold_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    plugin_json = target / "plugin.json"
    if not plugin_json.is_file():
        emit_error_and_exit(
            f"Scaffold failed: plugin.json not found in '{target}'.",
            code=1,
            failure="plugin_json_missing",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )
    try:
        meta = json.loads(plugin_json.read_text("utf-8"))
        if not (
            isinstance(meta, dict)
            and meta.get("name")
            and (meta.get("desc") or meta.get("description"))
        ):
            raise ValueError("Missing required fields")
    except Exception as exc:
        emit_error_and_exit(
            f"Scaffold failed: plugin.json invalid: {exc}",
            code=1,
            failure="plugin_json_invalid",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=verbose,
            debug=debug,
        )

    payload = {"status": "created", "plugin": name, "dir": str(target)}

    new_run_command(
        command_name=command,
        payload_builder=lambda include: payload,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
