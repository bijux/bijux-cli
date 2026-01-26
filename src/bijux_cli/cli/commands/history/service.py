# SPDX-License-Identifier: Apache-2.0
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
    * Error:          `{"error": str, "code": int}`

Exit Codes:
    * `0`: Success.
    * `1`: A fatal error occurred (e.g., history service unavailable).
    * `2`: An invalid argument was provided or an I/O error occurred during
      import/export.
"""

from __future__ import annotations

from dataclasses import dataclass
import json
from pathlib import Path
import platform
from typing import Any

import typer

from bijux_cli.cli.core.command import (
    ascii_safe,
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
from bijux_cli.core.di import DIContainer
from bijux_cli.core.enums import ErrorType, LogLevel, OutputFormat
from bijux_cli.core.precedence import current_execution_policy
from bijux_cli.services.history.contracts import HistoryProtocol


def resolve_history_service(
    command: str,
    fmt_lower: OutputFormat,
    quiet: bool,
    include_runtime: bool,
    log_level: LogLevel,
) -> HistoryProtocol:
    """Resolves the HistoryProtocol implementation from the DI container.

    Args:
        command (str): The full command name (e.g., "history").
        fmt_lower (OutputFormat): The chosen output format.
        quiet (bool): If True, suppresses non-error output.
        include_runtime (bool): If True, includes runtime metadata in errors.
        log_level (LogLevel): Logging level for diagnostics.

    Returns:
        HistoryProtocol: An instance of the history service.

    Raises:
        SystemExit: Exits with a structured error if the service cannot be
            resolved from the container.
    """
    try:
        return DIContainer.current().resolve(HistoryProtocol)
    except Exception as exc:
        raise_exit_intent(
            f"History service unavailable: {exc}",
            code=1,
            failure="service_unavailable",
            error_type=ErrorType.INTERNAL,
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
        )


@dataclass(frozen=True)
class HistoryIntent:
    """Resolved intent for the history command."""

    command: str
    action: str
    limit: int
    group_by: str | None
    filter_cmd: str | None
    sort: str | None
    export_path: str | None
    import_path: str | None
    quiet: bool
    include_runtime: bool
    log_level: LogLevel
    fmt: OutputFormat


def _build_history_intent(
    *,
    command: str,
    limit: int,
    group_by: str | None,
    filter_cmd: str | None,
    sort: str | None,
    export_path: str | None,
    import_path: str | None,
    fmt_lower: OutputFormat,
    quiet: bool,
    include_runtime: bool,
    log_level: LogLevel,
) -> HistoryIntent:
    """Validate inputs and build a history intent."""
    action = "list"
    if import_path:
        action = "import"
    elif export_path:
        action = "export"

    if limit < 0:
        raise_exit_intent(
            "Invalid value for --limit: must be non-negative.",
            code=2,
            failure="limit",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )

    if sort and sort != "timestamp":
        raise_exit_intent(
            "Invalid sort key: only 'timestamp' is supported.",
            code=2,
            failure="sort",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )

    if group_by and group_by != "command":
        raise_exit_intent(
            "Invalid group_by: only 'command' is supported.",
            code=2,
            failure="group_by",
            command=command,
            fmt=fmt_lower,
            quiet=quiet,
            include_runtime=include_runtime,
            log_level=log_level,
            error_type=ErrorType.USER_INPUT,
        )

    return HistoryIntent(
        command=command,
        action=action,
        limit=limit,
        group_by=group_by,
        filter_cmd=filter_cmd,
        sort=sort,
        export_path=export_path,
        import_path=import_path,
        quiet=quiet,
        include_runtime=include_runtime,
        log_level=log_level,
        fmt=fmt_lower,
    )


def _import_history(
    intent: HistoryIntent, history_svc: HistoryProtocol
) -> dict[str, object]:
    """Import history data and return a payload."""
    try:
        text = Path(intent.import_path or "").read_text(encoding="utf-8").strip()
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
        raise_exit_intent(
            f"Failed to import history: {exc}",
            code=2,
            failure="import_failed",
            command=intent.command,
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
            error_type=ErrorType.USER_INPUT,
        )

    payload: dict[str, object] = {
        "status": "imported",
        "file": intent.import_path or "",
    }
    if intent.include_runtime:
        return {
            "status": payload["status"],
            "file": payload["file"],
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def _export_history(
    intent: HistoryIntent, history_svc: HistoryProtocol
) -> dict[str, object]:
    """Export history data and return a payload."""
    try:
        entries = history_svc.list()
        from bijux_cli.cli.core.command import resolve_serializer

        rendered = resolve_serializer().dumps(entries, fmt=intent.fmt, pretty=True)
        Path(intent.export_path or "").write_text(
            rendered.rstrip("\n") + "\n",
            encoding="utf-8",
        )
    except Exception as exc:
        raise_exit_intent(
            f"Failed to export history: {exc}",
            code=2,
            failure="export_failed",
            command=intent.command,
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
            error_type=ErrorType.USER_INPUT,
        )

    payload: dict[str, object] = {
        "status": "exported",
        "file": intent.export_path or "",
    }
    if intent.include_runtime:
        return {
            "status": payload["status"],
            "file": payload["file"],
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def _list_history(
    intent: HistoryIntent, history_svc: HistoryProtocol
) -> dict[str, object]:
    """List history entries and return a payload."""
    try:
        entries = history_svc.list()
        if intent.filter_cmd:
            entries = [e for e in entries if intent.filter_cmd in e.get("command", "")]
        if intent.sort == "timestamp":
            entries = sorted(entries, key=lambda e: e.get("timestamp", 0))
        if intent.group_by == "command":
            groups: dict[str, list[dict[str, Any]]] = {}
            for e in entries:
                groups.setdefault(e.get("command", ""), []).append(e)
            entries = [
                {"group": k, "count": len(v), "entries": v} for k, v in groups.items()
            ]
        if intent.limit == 0:
            entries = []
        elif intent.limit > 0:
            entries = entries[-intent.limit :]
    except Exception as exc:
        raise_exit_intent(
            f"Failed to list history: {exc}",
            code=1,
            failure="list_failed",
            command=intent.command,
            fmt=intent.fmt,
            quiet=intent.quiet,
            include_runtime=intent.include_runtime,
            log_level=intent.log_level,
        )

    payload: dict[str, object] = {"entries": entries}
    if intent.include_runtime:
        return {
            "entries": payload["entries"],
            "python": ascii_safe(platform.python_version(), "python_version"),
            "platform": ascii_safe(platform.platform(), "platform"),
        }
    return payload


def _with_runtime(
    payload: dict[str, object], include_runtime: bool
) -> dict[str, object]:
    """Attach runtime metadata when requested."""
    if not include_runtime:
        return payload
    return {
        **payload,
        "python": ascii_safe(platform.python_version(), "python_version"),
        "platform": ascii_safe(platform.platform(), "platform"),
    }


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
    quiet: bool = typer.Option(False, *OPT_QUIET, help=HELP_QUIET),
    fmt: str = typer.Option("json", *OPT_FORMAT, help=HELP_FORMAT),
    pretty: bool = typer.Option(True, OPT_PRETTY, help=HELP_NO_PRETTY),
    log_level: str = typer.Option("info", *OPT_LOG_LEVEL, help=HELP_LOG_LEVEL),
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
        fmt (str): The output format ("json" or "yaml").
        pretty (bool): If True, pretty-prints the output.
        log_level (str): Logging level for diagnostics.

    Returns:
        None:

    Raises:
        SystemExit: Always exits with a contract-compliant status code and
            payload upon completion or error.
    """
    if ctx.invoked_subcommand:
        return

    command = "history"
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

    history_svc = resolve_history_service(
        command, fmt_lower, quiet, include_runtime, log_level_value
    )

    intent = _build_history_intent(
        command=command,
        limit=limit,
        group_by=group_by,
        filter_cmd=filter_cmd,
        sort=sort,
        export_path=export_path,
        import_path=import_path,
        fmt_lower=fmt_lower,
        quiet=quiet,
        include_runtime=include_runtime,
        log_level=log_level_value,
    )

    payload: dict[str, object]
    if intent.action == "import":
        payload = _import_history(intent, history_svc)
    elif intent.action == "export":
        payload = _export_history(intent, history_svc)
    else:
        payload = _list_history(intent, history_svc)

    new_run_command(
        command_name=command,
        payload_builder=lambda include_runtime: _with_runtime(payload, include_runtime),
        quiet=quiet,
        fmt=fmt_lower,
        pretty=pretty,
        log_level=log_level_value,
    )
