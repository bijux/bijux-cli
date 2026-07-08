"""Rust-backed facade with subprocess fallback."""

from __future__ import annotations

from collections.abc import Iterable
import importlib
import json
import os
from pathlib import Path
import subprocess
import sys

from ._exceptions import (
    BijuxPythonError,
    InternalError,
    NativeExtensionUnavailable,
    PlatformWheelUnavailable,
    UsageError,
    ValidationError,
)
from ._runtime import (
    ExecutionResult,
    RuntimeResolution,
    classify_process_error_kind,
    resolve_runtime_binary,
    run_subprocess_runtime,
    runtime_binary_filenames,
    runtime_timeout_seconds,
    sanitized_subprocess_env,
    validate_binary_candidate,
    workspace_runtime_binaries,
)


def _strict_native_import_enabled() -> bool:
    configured = os.environ.get("BIJUX_PY_STRICT_IMPORT")
    if configured is not None:
        return configured.strip().lower() not in {
            "",
            "0",
            "false",
            "no",
            "off",
        }
    return os.environ.get("BIJUX_ENV", "").strip().lower() in {"dev", "test", "ci"}


_STRICT_NATIVE_IMPORT = _strict_native_import_enabled()
_NATIVE_IMPORT_ERROR: Exception | None = None
_COMPAT_CONFIG_ENV_KEYS = {
    "BIJUXCLI_CONFIG",
    "BIJUXCLI_HISTORY_FILE",
    "BIJUXCLI_PLUGINS_DIR",
}


def _is_missing_native_module(exc: ImportError | ModuleNotFoundError) -> bool:
    name = getattr(exc, "name", None)
    if isinstance(name, str) and (
        name == "bijux_cli_py._native" or name.endswith("._native")
    ):
        return True
    message = str(exc)
    if "No module named" in message and "bijux_cli_py._native" in message:
        return True
    return "cannot import name '_native'" in message and "bijux_cli_py" in message


def _allow_native_import_fallback(exc: Exception) -> bool:
    if isinstance(exc, ModuleNotFoundError):
        return _is_missing_native_module(exc)
    if isinstance(exc, ImportError):
        configured = os.environ.get("BIJUX_PY_ALLOW_NATIVE_OSERROR_FALLBACK", "")
        if configured.strip().lower() in {"1", "true", "yes", "on"}:
            return True
        return _is_missing_native_module(exc)
    if isinstance(exc, OSError):
        configured = os.environ.get("BIJUX_PY_ALLOW_NATIVE_OSERROR_FALLBACK", "")
        return configured.strip().lower() in {"1", "true", "yes", "on"}
    return False


def _runtime_binary_filenames() -> tuple[str, ...]:
    if os.name == "nt":
        return ("bijux.exe", "bijux")
    return ("bijux",)


def _sanitized_subprocess_env() -> dict[str, str]:
    return {
        key: value
        for key, value in os.environ.items()
        if key not in _SUBPROCESS_ENV_STRIP_KEYS
    }


try:
    native = importlib.import_module(f"{__package__}._native")

    NATIVE_AVAILABLE = True
except (ImportError, ModuleNotFoundError, OSError) as exc:  # pragma: no cover
    if _STRICT_NATIVE_IMPORT or not _allow_native_import_fallback(exc):
        raise
    native = None
    NATIVE_AVAILABLE = False
    _NATIVE_IMPORT_ERROR = exc


def _resolve_binary() -> RuntimeResolution:
    current_entrypoint = sys.argv[0] if sys.argv and sys.argv[0] else None
    return resolve_runtime_binary(
        binary_name="bijux",
        env_key="BIJUX_BIN",
        workspace_candidates=_workspace_runtime_binaries(),
        current_entrypoint=current_entrypoint,
    )


def _workspace_runtime_binaries() -> list[str]:
    return workspace_runtime_binaries(
        module_file=__file__,
        workspace_crate_dir="bijux-cli",
        binary_name="bijux",
    )


def version() -> str:
    result = execution_facade_with_status(["version"])
    if result.exit_code != 0:
        raise error_to_exception(
            {
                "error_kind": result.error_kind,
                "message": result.stderr or "failed to query version",
            }
        )
    return result.stdout.strip()


def command_tree_introspection() -> str:
    if NATIVE_AVAILABLE:
        return native.command_tree_introspection()
    result = execution_facade_with_status(
        ["inspect", "--format", "json", "--no-pretty"]
    )
    if result.exit_code != 0:
        return json.dumps(
            {
                "root": "bijux",
                "namespaces": [],
                "source": "fallback-empty",
                "warning": f"inspect command failed: {result.stderr.strip() or f'exit {result.exit_code}'}",
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
    return json.dumps(
        {
            "root": "bijux",
            "namespaces": normalized,
            "source": "runtime-inspect",
        }
    )


def execution_facade(argv: Iterable[str]) -> str:
    result = execution_facade_with_status(argv)
    return result.stdout if result.exit_code == 0 else result.stderr


def execution_facade_with_status(argv: Iterable[str]) -> ExecutionResult:
    args = list(argv)
    if args in (["--version"], ["-V"]):
        args = ["version"]

    if NATIVE_AVAILABLE:
        if not hasattr(native, "execution_outcome"):
            raise NativeExtensionUnavailable(
                "Rust extension missing `execution_outcome`; reinstall bijux-cli to restore runtime parity."
            )
        try:
            outcome = json.loads(native.execution_outcome(args))
        except json.JSONDecodeError as exc:
            raise InternalError(
                f"invalid native execution outcome payload: {exc}"
            ) from exc
        return ExecutionResult(
            exit_code=int(outcome.get("exit_code", 1)),
            stdout=str(outcome.get("stdout", "")),
            stderr=str(outcome.get("stderr", "")),
            error_kind=(
                _normalize_error_kind(outcome.get("error_kind"))
                or _classify_process_error_kind(
                    int(outcome.get("exit_code", 1)),
                    str(outcome.get("stderr", "")),
                )
            ),
        )

    runtime = _resolve_binary()
    return run_subprocess_runtime(
        binary=runtime.binary,
        args=args,
        timeout_seconds=_runtime_timeout_seconds(),
        classify_error_kind=_classify_process_error_kind,
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
        if not hasattr(native, "config_resolution"):
            raise NativeExtensionUnavailable(
                "Rust extension missing `config_resolution`; reinstall bijux-cli to restore runtime parity."
            )
        try:
            payload = json.loads(native.config_resolution(home_dir))
        except json.JSONDecodeError as exc:
            raise InternalError(
                f"invalid native config resolution payload: {exc}"
            ) from exc
        except RuntimeError as exc:
            raise InternalError(f"native config resolution failed: {exc}") from exc
        return {
            "config_file": str(payload.get("config_file", "")),
            "history_file": str(payload.get("history_file", "")),
            "plugins_dir": str(payload.get("plugins_dir", "")),
        }
    try:
        return _resolve_config_paths_without_native(home_dir)
    except (OSError, ValueError) as exc:
        raise InternalError(f"failed to resolve config paths: {exc}") from exc


def plugin_registry_inspection(registry_file: str) -> dict[str, object]:
    if NATIVE_AVAILABLE:
        try:
            payload = json.loads(native.plugin_registry_inspection(registry_file))
        except json.JSONDecodeError as exc:
            raise InternalError(
                f"invalid plugin registry payload from native bridge: {exc}"
            ) from exc
        except RuntimeError as exc:
            raise InternalError(
                f"native plugin registry inspection failed: {exc}"
            ) from exc
        if not isinstance(payload, dict):
            raise InternalError("plugin registry payload must be a JSON object")
        return payload
    path = Path(registry_file)
    if not path.exists():
        return {"version": "1", "plugins": {}}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise InternalError(f"failed to read plugin registry: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise InternalError(f"invalid plugin registry json: {exc}") from exc
    if not isinstance(payload, dict):
        raise InternalError("plugin registry payload must be a JSON object")
    return payload


def install_path_helpers(home_dir: str) -> dict[str, str]:
    return config_resolution_helpers(home_dir)


def ensure_native_extension() -> None:
    if not NATIVE_AVAILABLE:
        raise NativeExtensionUnavailable(
            f"Rust extension failed to load: {_NATIVE_IMPORT_ERROR!r}"
        )


def check_python_runtime_supported(version_info: tuple[int, int] | None = None) -> bool:
    major, minor = version_info or (sys.version_info.major, sys.version_info.minor)
    return (major, minor) >= (3, 11)


def migration_warnings(legacy_python_only: bool = False) -> list[str]:
    if not legacy_python_only:
        return []
    return [
        "Python-only runtime assumptions are deprecated; commands delegate to Rust runtime."
    ]


def post_install_diagnostics() -> dict[str, object]:
    warnings_out: list[str] = []
    diagnostics = {
        "runtime_supported": check_python_runtime_supported(),
        "native_extension_available": NATIVE_AVAILABLE,
        "runtime_binary": None,
        "warnings": warnings_out,
    }
    try:
        diagnostics["runtime_binary"] = _resolve_binary().binary
    except PlatformWheelUnavailable as error:
        warnings_out.append(str(error))
    return diagnostics


def _normalize_error_kind(value: object) -> str | None:
    if isinstance(value, str) and value in {
        "UsageError",
        "ValidationError",
        "InternalError",
    }:
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
    return classify_process_error_kind(exit_code, stderr)


def _runtime_timeout_seconds() -> int:
    return runtime_timeout_seconds()


def _validate_binary_candidate(candidate: str, source: str) -> str:
    return validate_binary_candidate(candidate, source)


def _runtime_binary_filenames() -> tuple[str, ...]:
    return runtime_binary_filenames("bijux")


def _sanitized_subprocess_env() -> dict[str, str]:
    return sanitized_subprocess_env()


def _resolve_config_paths_without_native(home_dir: str) -> dict[str, str]:
    home = Path(home_dir).expanduser()
    base = home / ".bijux"
    defaults = {
        "config_file": base / ".env",
        "history_file": base / ".history",
        "plugins_dir": base / ".plugins",
    }
    config_overrides = _load_compatibility_overrides(defaults["config_file"])

    return {
        "config_file": str(
            _select_compatibility_path(
                "BIJUXCLI_CONFIG",
                config_overrides.get("BIJUXCLI_CONFIG"),
                defaults["config_file"],
                home,
            )
        ),
        "history_file": str(
            _select_compatibility_path(
                "BIJUXCLI_HISTORY_FILE",
                config_overrides.get("BIJUXCLI_HISTORY_FILE"),
                defaults["history_file"],
                home,
            )
        ),
        "plugins_dir": str(
            _select_compatibility_path(
                "BIJUXCLI_PLUGINS_DIR",
                config_overrides.get("BIJUXCLI_PLUGINS_DIR"),
                defaults["plugins_dir"],
                home,
            )
        ),
    }


def _load_compatibility_overrides(path: Path) -> dict[str, str]:
    if not path.exists():
        return {}
    text = path.read_text(encoding="utf-8")
    parsed: dict[str, str] = {}
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError(
                f"malformed compatibility config line {line_number} in {path}: {line!r}"
            )
        key, value = line.split("=", 1)
        normalized_key = key.strip()
        if normalized_key not in _COMPAT_CONFIG_ENV_KEYS:
            raise ValueError(
                "unsupported compatibility config key "
                f"{normalized_key!r} in {path}; expected one of "
                f"{sorted(_COMPAT_CONFIG_ENV_KEYS)}"
            )
        parsed[normalized_key] = value.strip()
    return parsed


def _select_compatibility_path(
    env_key: str, config_value: str | None, default_value: Path, home_dir: Path
) -> Path:
    candidate = os.environ.get(env_key) or config_value or str(default_value)
    return _normalize_compatibility_path(Path(candidate).expanduser(), home_dir)


def _normalize_compatibility_path(path: Path, home_dir: Path) -> Path:
    if path.is_absolute():
        return path
    return home_dir / path
