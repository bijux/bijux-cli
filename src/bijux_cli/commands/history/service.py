# SPDX-License-Identifier: MIT
# Copyright © 2025 Bijan Mousavi

"""Implements the `history` command for the Bijux CLI.

This module provides functionality to interact with the persistent command
history. It allows for listing, filtering, sorting, grouping, importing, and
exporting history entries. All operations produce structured, machine-readable
output.

The command has three primary modes of operation:
1.  **Listing (Default):** When no import/export flags are used, it lists
    history entries, which can be filtered, sorted, and grouped.
2.  **Import:** The `--import` flag replaces the current history with data
    from a specified JSON file.
3.  **Export:** The `--export` flag writes the entire current history to a
    specified JSON file.

Output Contract:
    * List Success:   `{"entries": list}`
    * Import Success: `{"status": "imported", "file": str}`
    * Export Success: `{"status": "exported", "file": str}`
    * Verbose:        Adds `{"python": str, "platform": str}` to the payload.
    * Error:          `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal error occurred (e.g., history service unavailable).
    * `2`: An invalid argument was provided or an I/O error occurred during
      import/export.
"""

from __future__ import annotations

from collections.abc import Mapping
import json
from pathlib import Path
import platform
from typing import Any

import typer

from bijux_cli.commands.utilities import (
    ascii_safe,
    emit_error_and_exit,
    new_run_command,
    validate_common_flags,
)
from bijux_cli.contracts import HistoryProtocol
from bijux_cli.core.constants import (
    HELP_DEBUG,
    HELP_FORMAT,
    HELP_NO_PRETTY,
    HELP_QUIET,
    HELP_VERBOSE,
)
from bijux_cli.core.di import DIContainer


def resolve_history_service(
    command: str, fmt_lower: str, quiet: bool, include_runtime: bool, debug: bool
) -> HistoryProtocol:
    """Resolves the HistoryProtocol implementation from the DI container.

    Args:
        command (str): The full command name (e.g., "history").
        fmt_lower (str): The chosen output format, lowercased.
        quiet (bool): If True, suppresses non-error output.
        include_runtime (bool): If True, includes runtime metadata in errors.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        HistoryProtocol: An instance of the history service.

    Raises:
        SystemExit: Exits with a structured error if the service cannot be
            resolved from the container.
    """
    try:
        return DIContainer.current().resolve(HistoryProtocol)
    except Exception as exc:
        emit_error_and_exit(
            f"History service unavailable: {exc}",
            code=1,
            failure="service_unavailable",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )


def history(
    ctx: typer.Context,
    limit: int = typer.Option(
        20, "--limit", "-l", help="Maximum number of entries (0 means none)."
    ),
    group_by: str | None = typer.Option(
        None, "--group-by", "-g", help="Group entries by a field (e.g., 'command')."
    ),
    filter_cmd: str | None = typer.Option(
        None, "--filter", "-F", help="Return only entries whose command contains TEXT."
    ),
    sort: str | None = typer.Option(
        None, "--sort", help="Sort key; currently only 'timestamp' is recognized."
    ),
    export_path: str = typer.Option(
        None, "--export", help="Write entire history to FILE (JSON). Overwrites."
    ),
    import_path: str = typer.Option(
        None, "--import", help="Load history from FILE (JSON), replacing current store."
    ),
    quiet: bool = typer.Option(False, "-q", "--quiet", help=HELP_QUIET),
    verbose: bool = typer.Option(False, "-v", "--verbose", help=HELP_VERBOSE),
    fmt: str = typer.Option("json", "-f", "--format", help=HELP_FORMAT),
    pretty: bool = typer.Option(True, "--pretty/--no-pretty", help=HELP_NO_PRETTY),
    debug: bool = typer.Option(False, "-d", "--debug", help=HELP_DEBUG),
) -> None:
    """Lists, imports, or exports the command history.

    This function orchestrates all history-related operations. It first checks
    for an import or export action. If neither is specified, it proceeds to
    list the history, applying any specified filtering, grouping, or sorting.

    Args:
        ctx (typer.Context): The Typer context for the CLI.
        limit (int): The maximum number of entries to return for a list operation.
        group_by (str | None): The field to group history entries by ('command').
        filter_cmd (str | None): A substring to filter command names by.
        sort (str | None): The key to sort entries by ('timestamp').
        export_path (str): The path to export history to. This is an exclusive action.
        import_path (str): The path to import history from. This is an exclusive action.
        quiet (bool): If True, suppresses all output except for errors.
        verbose (bool): If True, includes Python/platform details in the output.
        fmt (str): The output format ("json" or "yaml").
        pretty (bool): If True, pretty-prints the output.
        debug (bool): If True, enables debug diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    if ctx.invoked_subcommand:
        return

    command = "history"
    if debug:
        verbose = True
        pretty = True
    include_runtime = verbose

    fmt_lower = validate_common_flags(
        fmt,
        command,
        quiet,
        include_runtime=include_runtime,
    )

    history_svc = resolve_history_service(
        command, fmt_lower, quiet, include_runtime, debug
    )

    if limit < 0:
        emit_error_and_exit(
            "Invalid value for --limit: must be non-negative.",
            code=2,
            failure="limit",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    if sort and sort != "timestamp":
        emit_error_and_exit(
            "Invalid sort key: only 'timestamp' is supported.",
            code=2,
            failure="sort",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    if group_by and group_by != "command":
        emit_error_and_exit(
            "Invalid group_by: only 'command' is supported.",
            code=2,
            failure="group_by",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    if import_path:
        try:
            text = Path(import_path).read_text(encoding="utf-8").strip()
            data = json.loads(text or "[]")
            if not isinstance(data, list):
                raise ValueError("Import file must contain a JSON array.")
            history_svc.clear()
            for item in data:
                if not isinstance(item, dict):
                    continue
                cmd = str(item.get("command") or item.get("cmd", ""))
                cmd = ascii_safe(cmd, "command")
                if not cmd:
                    continue
                history_svc.add(
                    command=cmd,
                    params=item.get("params", []),
                    success=bool(item.get("success", True)),
                    return_code=item.get("return_code", 0),
                    duration_ms=item.get("duration_ms", 0.0),
                )
        except Exception as exc:
            emit_error_and_exit(
                f"Failed to import history: {exc}",
                code=2,
                failure="import_failed",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                debug=debug,
            )

        def payload_builder(_: bool) -> Mapping[str, Any]:
            """Builds the payload confirming a successful import.

            Args:
                _ (bool): Unused parameter to match the expected signature.

            Returns:
                Mapping[str, Any]: The structured payload.
            """
            payload: dict[str, Any] = {"status": "imported", "file": import_path}
            if include_runtime:
                payload["python"] = ascii_safe(
                    platform.python_version(), "python_version"
                )
                payload["platform"] = ascii_safe(platform.platform(), "platform")
            return payload

        new_run_command(
            command_name=command,
            payload_builder=payload_builder,
            quiet=quiet,
            verbose=verbose,
            fmt=fmt_lower,
            pretty=pretty,
            debug=debug,
        )

    if export_path:
        try:
            entries = history_svc.list()
            Path(export_path).write_text(
                json.dumps(entries, indent=2 if pretty else None) + "\n",
                encoding="utf-8",
            )
        except Exception as exc:
            emit_error_and_exit(
                f"Failed to export history: {exc}",
                code=2,
                failure="export_failed",
                command=command,
                fmt=fmt_lower,
                quiet=quiet,
                include_runtime=include_runtime,
                debug=debug,
            )

        def payload_builder(_: bool) -> Mapping[str, Any]:
            """Builds the payload confirming a successful export.

            Args:
                _ (bool): Unused parameter to match the expected signature.

            Returns:
                Mapping[str, Any]: The structured payload.
            """
            payload: dict[str, Any] = {"status": "exported", "file": export_path}
            if include_runtime:
                payload["python"] = ascii_safe(
                    platform.python_version(), "python_version"
                )
                payload["platform"] = ascii_safe(platform.platform(), "platform")
            return payload

        new_run_command(
            command_name=command,
            payload_builder=payload_builder,
            quiet=quiet,
            verbose=verbose,
            fmt=fmt_lower,
            pretty=pretty,
            debug=debug,
        )

    try:
        entries = history_svc.list()
        if filter_cmd:
            entries = [e for e in entries if filter_cmd in e.get("command", "")]
        if sort == "timestamp":
            entries = sorted(entries, key=lambda e: e.get("timestamp", 0))
        if group_by == "command":
            groups: dict[str, list[dict[str, Any]]] = {}
            for e in entries:
                groups.setdefault(e.get("command", ""), []).append(e)
            entries = [
                {"group": k, "count": len(v), "entries": v} for k, v in groups.items()
            ]
        if limit == 0:
            entries = []
        elif limit > 0:
            entries = entries[-limit:]

    except Exception as exc:
        emit_error_and_exit(
            f"Failed to list history: {exc}",
            code=1,
            failure="list_failed",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            debug=debug,
        )

    def list_payload_builder(include_runtime: bool) -> Mapping[str, Any]:
        """Builds the payload containing a list of history entries.

        Args:
            include_runtime (bool): If True, includes Python and platform info.

        Returns:
            Mapping[str, Any]: The structured payload.
        """
        payload: dict[str, Any] = {"entries": entries}
        if include_runtime:
            payload["python"] = ascii_safe(platform.python_version(), "python_version")
            payload["platform"] = ascii_safe(platform.platform(), "platform")
        return payload

    new_run_command(
        command_name=command,
        payload_builder=list_payload_builder,
        quiet=quiet,
        verbose=verbose,
        fmt=fmt_lower,
        pretty=pretty,
        debug=debug,
    )
