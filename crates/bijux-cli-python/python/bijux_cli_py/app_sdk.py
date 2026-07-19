"""Helpers for authoring mounted Python apps for the Bijux root CLI."""

from __future__ import annotations

from collections.abc import Callable, Iterable, Sequence
from contextlib import redirect_stdout
from dataclasses import dataclass
from datetime import UTC, datetime
import io
import json
import sys
import traceback
from typing import Any


def _normalize_segments(command: Sequence[str] | None) -> list[str]:
    if not command:
        return ["app"]
    normalized: list[str] = []
    for segment in command:
        candidate = (
            segment.strip()
            .lower()
            .replace("_", "-")
            .replace("/", "-")
            .replace(" ", "-")
        )
        candidate = "-".join(part for part in candidate.split("-") if part)
        if not candidate:
            raise ValueError("command segments must not be empty")
        normalized.append(candidate)
    return normalized


def _timestamp(value: str | None = None) -> str:
    if value:
        return value
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")


def _semver_tuple(value: str) -> tuple[int, int, int]:
    core = value.strip().split("-", 1)[0].split("+", 1)[0]
    parts = core.split(".")
    if len(parts) < 3:
        raise ValueError(f"invalid semver: {value}")
    return (int(parts[0]), int(parts[1]), int(parts[2]))


@dataclass(frozen=True)
class CompatibilityWindow:
    min_cli_version: str
    max_cli_version_exclusive: str | None = None

    def report(self, host_cli_version: str) -> dict[str, Any]:
        host = _semver_tuple(host_cli_version)
        minimum = _semver_tuple(self.min_cli_version)
        maximum = (
            _semver_tuple(self.max_cli_version_exclusive)
            if self.max_cli_version_exclusive
            else None
        )
        reasons: list[str] = []
        if host < minimum:
            reasons.append(
                f"host version `{host_cli_version}` is below required minimum `{self.min_cli_version}`"
            )
        if maximum is not None and host >= maximum:
            reasons.append(
                "host version "
                f"`{host_cli_version}` is not below exclusive maximum `{self.max_cli_version_exclusive}`"
            )
        return {
            "compatible": not reasons,
            "host_cli_version": host_cli_version,
            "min_cli_version": self.min_cli_version,
            "max_cli_version_exclusive": self.max_cli_version_exclusive,
            "reasons": reasons,
        }


@dataclass(frozen=True)
class CommandResult:
    exit_code: int
    stdout: str
    stderr: str = ""

    def emit(self) -> int:
        if self.stdout:
            sys.stdout.write(self.stdout)
        if self.stderr:
            sys.stderr.write(self.stderr)
        return self.exit_code


def success(
    data: Any,
    *,
    command: Sequence[str] | None = None,
    timestamp: str | None = None,
) -> CommandResult:
    payload = {
        "status": "ok",
        "data": data,
        "meta": {
            "version": "v1",
            "command": {"segments": _normalize_segments(command)},
            "timestamp": _timestamp(timestamp),
        },
    }
    return CommandResult(
        exit_code=0,
        stdout=f"{json.dumps(payload, indent=2)}\n",
    )


def failure(
    code: str,
    message: str,
    *,
    command: Sequence[str] | None = None,
    category: str = "internal",
    details: dict[str, Any] | None = None,
    exit_code: int = 1,
    timestamp: str | None = None,
) -> CommandResult:
    payload = {
        "status": "error",
        "error": {
            "code": code,
            "message": message,
            "category": category,
            "details": details or None,
        },
        "meta": {
            "version": "v1",
            "command": {"segments": _normalize_segments(command)},
            "timestamp": _timestamp(timestamp),
        },
    }
    return CommandResult(
        exit_code=exit_code,
        stderr=f"{json.dumps(payload, indent=2)}\n",
    )


def compatibility_report(
    min_cli_version: str,
    max_cli_version_exclusive: str | None = None,
    *,
    host_cli_version: str = "0.4.0",
) -> dict[str, Any]:
    return CompatibilityWindow(
        min_cli_version=min_cli_version,
        max_cli_version_exclusive=max_cli_version_exclusive,
    ).report(host_cli_version)


def build_python_mount_manifest(
    *,
    namespace: str,
    display_name: str,
    module: str,
    function: str = "main",
    summary: str,
    aliases: Sequence[str] | None = None,
    capabilities: Sequence[str] | None = None,
    version: str | None = None,
    compatibility: CompatibilityWindow | None = None,
) -> dict[str, Any]:
    manifest: dict[str, Any] = {
        "namespace": namespace,
        "display_name": display_name,
        "aliases": list(aliases or []),
        "entrypoint": {
            "kind": "python_module",
            "command": module,
            "module": module,
            "function": function,
        },
        "control_entrypoint": {
            "kind": "python_module",
            "command": module,
            "module": module,
            "function": function,
        },
        "help": {"summary": summary},
        "capabilities": list(capabilities or []),
    }
    if version is not None:
        manifest["version"] = version
    if compatibility is not None:
        manifest["compatibility"] = {
            "min_cli_version": compatibility.min_cli_version,
            "max_cli_version_exclusive": compatibility.max_cli_version_exclusive,
        }
    return manifest


def run_json_app(
    main: Callable[[list[str]], Any],
    *,
    argv: Iterable[str] | None = None,
    command: Sequence[str] | None = None,
    timestamp: str | None = None,
) -> int:
    args = list(argv or [])
    log_buffer = io.StringIO()
    try:
        with redirect_stdout(log_buffer):
            result = main(args)
    except Exception as exc:
        logs = log_buffer.getvalue()
        if logs:
            sys.stderr.write(logs)
        return failure(
            "python_app_exception",
            str(exc),
            command=command,
            details={
                "exception": type(exc).__name__,
                "traceback": traceback.format_exc(),
            },
            timestamp=timestamp,
        ).emit()

    logs = log_buffer.getvalue()
    if logs:
        sys.stderr.write(logs)

    if isinstance(result, CommandResult):
        return result.emit()
    if result is None:
        return success({}, command=command, timestamp=timestamp).emit()
    if isinstance(result, bool):
        return 0 if result else 1
    if isinstance(result, int):
        return result
    if isinstance(result, str):
        sys.stdout.write(result)
        return 0
    return success(result, command=command, timestamp=timestamp).emit()
