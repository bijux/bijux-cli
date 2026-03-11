"""Rust-backed facade with subprocess fallback."""

from __future__ import annotations

import os
import json
import shutil
import subprocess
import sys
from pathlib import Path
from dataclasses import dataclass
from importlib import metadata
from typing import Iterable

from ._exceptions import (
    BijuxPythonError,
    InternalError,
    NativeExtensionUnavailable,
    PlatformWheelUnavailable,
    UsageError,
    ValidationError,
)


_STRICT_NATIVE_IMPORT = os.environ.get("BIJUX_PY_STRICT_IMPORT") == "1"
_NATIVE_IMPORT_ERROR: Exception | None = None

try:
    from . import _native as native
    NATIVE_AVAILABLE = True
except (ImportError, ModuleNotFoundError, OSError) as exc:  # pragma: no cover
    if _STRICT_NATIVE_IMPORT:
        raise
    native = None
    NATIVE_AVAILABLE = False
    _NATIVE_IMPORT_ERROR = exc


@dataclass(frozen=True)
class RuntimeResolution:
    binary: str


@dataclass(frozen=True)
class ExecutionResult:
    exit_code: int
    stdout: str
    stderr: str
    error_kind: str | None = None


def _resolve_binary() -> RuntimeResolution:
    candidate = os.environ.get("BIJUX_BIN")
    if candidate:
        return RuntimeResolution(binary=candidate)

    for name in ("bijux",):
        resolved = shutil.which(name)
        if resolved:
            return RuntimeResolution(binary=resolved)

    raise PlatformWheelUnavailable(
        "No compatible runtime binary found. Set BIJUX_BIN or install bijux-cli wheel for this platform."
    )


def version() -> str:
    if NATIVE_AVAILABLE:
        return native.version()
    for package_name in ("bijux-cli", "bijux_cli_py"):
        try:
            return metadata.version(package_name)
        except metadata.PackageNotFoundError:
            continue
    result = execution_facade_with_status(["version"])
    if result.exit_code != 0:
        raise error_to_exception(
            {"error_kind": result.error_kind, "message": result.stderr or "failed to query version"}
        )
    return result.stdout.strip()


def command_tree_introspection() -> str:
    if NATIVE_AVAILABLE:
        return native.command_tree_introspection()
    result = execution_facade_with_status(["inspect", "--format", "json", "--no-pretty"])
    if result.exit_code != 0:
        return json.dumps(
            {
                "root": "bijux",
                "namespaces": [
                    "cli",
                    "dev",
                    "help",
                    "version",
                    "doctor",
                    "repl",
                    "plugins",
                    "completion",
                    "inspect",
                ],
            }
        )
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise InternalError(f"invalid inspect payload from runtime: {exc}") from exc

    namespaces: list[str] = []
    builtins = payload.get("builtins")
    if isinstance(builtins, list):
        for entry in builtins:
            if not isinstance(entry, dict):
                continue
            segments = entry.get("segments")
            if isinstance(segments, list) and segments:
                head = segments[0]
                if isinstance(head, str):
                    namespaces.append(head)

    normalized = sorted(set(namespaces))
    return json.dumps({"root": "bijux", "namespaces": normalized})


def execution_facade(argv: Iterable[str]) -> str:
    result = execution_facade_with_status(argv)
    return result.stdout if result.exit_code == 0 else result.stderr


def execution_facade_with_status(argv: Iterable[str]) -> ExecutionResult:
    args = list(argv)
    if NATIVE_AVAILABLE:
        if not hasattr(native, "execution_outcome"):
            raise NativeExtensionUnavailable(
                "Rust extension missing `execution_outcome`; reinstall bijux-cli to restore runtime parity."
            )
        try:
            outcome = json.loads(native.execution_outcome(args))
        except json.JSONDecodeError as exc:
            raise InternalError(f"invalid native execution outcome payload: {exc}") from exc
        return ExecutionResult(
            exit_code=int(outcome.get("exit_code", 1)),
            stdout=str(outcome.get("stdout", "")),
            stderr=str(outcome.get("stderr", "")),
            error_kind=_normalize_error_kind(outcome.get("error_kind")),
        )

    runtime = _resolve_binary()
    result = subprocess.run([runtime.binary, *args], capture_output=True, text=True, check=False)
    return ExecutionResult(
        exit_code=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
        error_kind=_classify_process_error_kind(result.returncode, result.stderr),
    )


def error_to_exception(payload: dict[str, object]) -> BijuxPythonError:
    message = _extract_error_message(payload)
    kind = _extract_error_kind(payload)
    if kind == "UsageError":
        return UsageError(message)
    if kind == "ValidationError":
        return ValidationError(message)
    if kind == "InternalError":
        return InternalError(message)
    return BijuxPythonError(message)


def config_resolution_helpers(home_dir: str) -> dict[str, str]:
    if NATIVE_AVAILABLE:
        payload = json.loads(native.install_paths(home_dir))
        return {
            "config_file": str(payload.get("config_file", "")),
            "history_file": str(payload.get("history_file", "")),
            "plugins_dir": str(payload.get("plugins_dir", "")),
        }
    base = Path(home_dir) / ".bijux"
    return {
        "config_file": str(base / ".env"),
        "history_file": str(base / ".history"),
        "plugins_dir": str(base / ".plugins"),
    }


def plugin_registry_inspection(registry_file: str) -> dict[str, object]:
    if NATIVE_AVAILABLE:
        return json.loads(native.plugin_registry_inspection(registry_file))
    path = Path(registry_file)
    if not path.exists():
        return {"version": "1", "plugins": {}}
    return json.loads(path.read_text(encoding="utf-8"))


def install_path_helpers(home_dir: str) -> dict[str, str]:
    return config_resolution_helpers(home_dir)


def ensure_native_extension() -> None:
    if not NATIVE_AVAILABLE:
        raise NativeExtensionUnavailable(
            f"Rust extension failed to load: {_NATIVE_IMPORT_ERROR!r}"
        )


def check_python_runtime_supported(version_info: tuple[int, int] | None = None) -> bool:
    major, minor = version_info or (sys.version_info.major, sys.version_info.minor)
    return (major, minor) >= (3, 9)


def migration_warnings(legacy_python_only: bool = False) -> list[str]:
    warnings_out: list[str] = []
    if legacy_python_only:
        warnings_out.append(
            "Python-only runtime assumptions are deprecated; commands now delegate to Rust-backed engine."
        )
    if os.environ.get("BIJUXCLI_PY_LEGACY") == "1":
        warnings_out.append("Detected legacy Python metadata mode; compatibility shims are active.")
    return warnings_out


def post_install_diagnostics() -> dict[str, object]:
    diagnostics = {
        "runtime_supported": check_python_runtime_supported(),
        "native_extension_available": NATIVE_AVAILABLE,
        "runtime_binary": None,
        "warnings": migration_warnings(),
    }
    try:
        diagnostics["runtime_binary"] = _resolve_binary().binary
    except PlatformWheelUnavailable as error:
        diagnostics["warnings"].append(str(error))
    return diagnostics


def _normalize_error_kind(value: object) -> str | None:
    if isinstance(value, str) and value in {"UsageError", "ValidationError", "InternalError"}:
        return value
    return None


def _extract_error_kind(payload: dict[str, object]) -> str | None:
    normalized = _normalize_error_kind(payload.get("error_kind"))
    if normalized:
        return normalized
    nested = payload.get("error")
    if isinstance(nested, dict):
        return _normalize_error_kind(nested.get("kind"))
    return None


def _extract_error_message(payload: dict[str, object]) -> str:
    if isinstance(payload.get("message"), str):
        return str(payload["message"])
    nested = payload.get("error")
    if isinstance(nested, dict) and isinstance(nested.get("message"), str):
        return str(nested["message"])
    return "Unknown bijux-cli error"


def _classify_process_error_kind(exit_code: int, stderr: str) -> str | None:
    if exit_code == 0:
        return None
    lower = stderr.lower()
    if "unknown route" in lower or "unknown namespace" in lower or "usage" in lower:
        return "UsageError"
    if "validation" in lower or "invalid" in lower:
        return "ValidationError"
    return "InternalError"
