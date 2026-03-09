"""Rust-backed facade with subprocess fallback."""

from __future__ import annotations

import os
import json
import shutil
import subprocess
import sys
import warnings
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


@dataclass(frozen=True)
class ExecutionResult:
    exit_code: int
    stdout: str
    stderr: str


@dataclass(frozen=True)
class PathAmbiguityReport:
    has_ambiguity: bool
    message: str


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
    result = execution_facade_with_status(argv)
    return result.stdout if result.exit_code == 0 else result.stderr


def execution_facade_with_status(argv: Iterable[str]) -> ExecutionResult:
    args = list(argv)
    if NATIVE_AVAILABLE:
        output = native.execution_facade(args)
        return ExecutionResult(exit_code=0, stdout=output, stderr="")

    runtime = _resolve_binary()
    result = subprocess.run([runtime.binary, *args], capture_output=True, text=True, check=False)
    return ExecutionResult(exit_code=result.returncode, stdout=result.stdout, stderr=result.stderr)


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


def check_embedded_binary_compatibility(expected_version: str) -> bool:
    actual = version()
    return actual == expected_version


def deprecated_version_api() -> str:
    warnings.warn(
        "deprecated_version_api is deprecated; use version() instead.",
        DeprecationWarning,
        stacklevel=2,
    )
    return version()


def path_ambiguity_detection_message(binaries: Iterable[str]) -> PathAmbiguityReport:
    resolved = [item for item in binaries if item]
    unique = sorted(set(resolved))
    if len(unique) <= 1:
        return PathAmbiguityReport(has_ambiguity=False, message="No PATH ambiguity detected.")

    return PathAmbiguityReport(
        has_ambiguity=True,
        message=(
            "Multiple bijux binaries detected in PATH order: "
            + ", ".join(unique)
            + ". Keep only one canonical entrypoint."
        ),
    )


def simulate_pip_uninstall_cleanup(install_root: str) -> dict[str, bool]:
    root = Path(install_root)
    removed = {"site_package_removed": False, "entrypoint_removed": False}

    for path in [root / "bijux_cli_py", root / "bin" / "bijux"]:
        if path.exists():
            if path.is_dir():
                for nested in sorted(path.rglob("*"), reverse=True):
                    if nested.is_file():
                        nested.unlink(missing_ok=True)
                    elif nested.is_dir():
                        nested.rmdir()
                path.rmdir()
            else:
                path.unlink()

        if path.name == "bijux_cli_py":
            removed["site_package_removed"] = not path.exists()
        if path.name == "bijux":
            removed["entrypoint_removed"] = not path.exists()

    return removed


def simulate_pip_upgrade_preserves_state(home_dir: str) -> dict[str, bool]:
    base = Path(home_dir) / ".bijux"
    expected = [base / ".env", base / ".history", base / ".plugins"]
    return {
        "config_preserved": expected[0].exists(),
        "history_preserved": expected[1].exists(),
        "plugins_preserved": expected[2].exists(),
    }


def side_by_side_install_report(
    pip_binary: str | None,
    cargo_binary: str | None,
) -> PathAmbiguityReport:
    return path_ambiguity_detection_message([pip_binary or "", cargo_binary or ""])
