"""Rust-backed facade with subprocess fallback."""

from __future__ import annotations

import os
import json
import shutil
import subprocess
from pathlib import Path
from dataclasses import dataclass
from typing import Iterable

from ._exceptions import (
    BijuxPythonError,
    NativeExtensionUnavailable,
    PlatformWheelUnavailable,
)


try:
    from . import _native as native
    NATIVE_AVAILABLE = True
except Exception as exc:  # pragma: no cover
    native = None
    NATIVE_AVAILABLE = False
    _NATIVE_IMPORT_ERROR = exc


@dataclass(frozen=True)
class RuntimeResolution:
    binary: str


def _resolve_binary() -> RuntimeResolution:
    candidate = os.environ.get("BIJUX_BIN")
    if candidate:
        return RuntimeResolution(binary=candidate)

    for name in ("bijux", "bijux-rs"):
        resolved = shutil.which(name)
        if resolved:
            return RuntimeResolution(binary=resolved)

    raise PlatformWheelUnavailable(
        "No compatible runtime binary found. Set BIJUX_BIN or install bijux-cli wheel for this platform."
    )


def version() -> str:
    if NATIVE_AVAILABLE:
        return native.version()
    return "0.1.0"


def command_tree_introspection() -> str:
    if NATIVE_AVAILABLE:
        return native.command_tree_introspection()
    return '{"root":"bijux"}'


def execution_facade(argv: Iterable[str]) -> str:
    args = list(argv)
    if NATIVE_AVAILABLE:
        return native.execution_facade(args)

    runtime = _resolve_binary()
    result = subprocess.run([runtime.binary, *args], capture_output=True, text=True, check=False)
    return result.stdout if result.returncode == 0 else result.stderr


def output_envelope_model() -> dict[str, object]:
    return {
        "status": "ok",
        "data": {"example": True},
        "meta": {"version": "v1", "command": {"segments": []}, "timestamp": "1970-01-01T00:00:00Z"},
    }


def error_to_exception(payload: dict[str, object]) -> BijuxPythonError:
    message = str(payload.get("message", "Unknown bijux-cli error"))
    return BijuxPythonError(message)


def config_resolution_helpers(home_dir: str) -> dict[str, str]:
    base = Path(home_dir) / ".bijux"
    return {
        "config_file": str(base / ".env"),
        "history_file": str(base / ".history"),
        "plugins_dir": str(base / ".plugins"),
    }


def plugin_registry_inspection(registry_file: str) -> dict[str, object]:
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
