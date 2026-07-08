"""Shared runtime-binary resolution and subprocess execution helpers."""

from __future__ import annotations

from collections.abc import Callable, Iterable
from dataclasses import dataclass
import os
from pathlib import Path
import shutil
import subprocess

from ._exceptions import PlatformWheelUnavailable

_SUBPROCESS_ENV_STRIP_KEYS = frozenset(
    {
        "PYTHONHOME",
        "PYTHONPATH",
        "PYTHONSTARTUP",
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
    }
)


@dataclass(frozen=True)
class RuntimeResolution:
    binary: str


@dataclass(frozen=True)
class ExecutionResult:
    exit_code: int
    stdout: str
    stderr: str
    error_kind: str | None = None


def runtime_binary_filenames(binary_name: str) -> tuple[str, ...]:
    if os.name == "nt":
        return (f"{binary_name}.exe", binary_name)
    return (binary_name,)


def sanitized_subprocess_env() -> dict[str, str]:
    return {
        key: value
        for key, value in os.environ.items()
        if key not in _SUBPROCESS_ENV_STRIP_KEYS
    }


def runtime_timeout_seconds(
    env_key: str = "BIJUX_PY_SUBPROCESS_TIMEOUT", default_seconds: int = 60
) -> int:
    configured = os.environ.get(env_key)
    if configured is None:
        return default_seconds
    try:
        parsed = int(configured)
    except ValueError:
        return default_seconds
    if parsed <= 0:
        return default_seconds
    return parsed


def validate_binary_candidate(candidate: str, source: str) -> str:
    candidate_path = Path(candidate).expanduser().resolve()
    if not candidate_path.is_file():
        raise PlatformWheelUnavailable(
            f"{source} binary does not exist: {candidate_path}"
        )
    if not os.access(candidate_path, os.X_OK):
        raise PlatformWheelUnavailable(
            f"{source} binary is not executable: {candidate_path}"
        )
    return str(candidate_path)


def workspace_runtime_binaries(
    *,
    module_file: str,
    workspace_crate_dir: str,
    binary_name: str,
) -> list[str]:
    module_path = Path(module_file).resolve()
    workspace_root = None
    for parent in module_path.parents:
        if (parent / "Cargo.toml").is_file() and (
            parent / "crates" / workspace_crate_dir
        ).is_dir():
            workspace_root = parent
            break
    if workspace_root is None:
        return []

    candidates: list[Path] = []
    for base in (
        workspace_root / "artifacts" / "rust" / "target" / "debug",
        workspace_root / "artifacts" / "rust" / "target" / "release",
        workspace_root / "target" / "debug",
        workspace_root / "target" / "release",
    ):
        candidates.extend(
            base / runtime_name
            for runtime_name in runtime_binary_filenames(binary_name)
        )
    return [str(path) for path in candidates]


def resolve_runtime_binary(
    *,
    binary_name: str,
    env_key: str,
    workspace_candidates: Iterable[str],
    current_entrypoint: str | None = None,
) -> RuntimeResolution:
    candidate = os.environ.get(env_key)
    if candidate:
        return RuntimeResolution(
            binary=validate_binary_candidate(candidate, env_key)
        )

    for resolved in workspace_candidates:
        try:
            return RuntimeResolution(
                binary=validate_binary_candidate(resolved, "workspace runtime")
            )
        except PlatformWheelUnavailable:
            continue

    current_entrypoint_path = None
    if current_entrypoint:
        current_entrypoint_path = Path(current_entrypoint).resolve()

    resolved = shutil.which(binary_name)
    if resolved:
        resolved_path = Path(resolved).resolve()
        if (
            current_entrypoint_path is None
            or resolved_path != current_entrypoint_path
        ):
            return RuntimeResolution(
                binary=validate_binary_candidate(str(resolved_path), "PATH")
            )

    raise PlatformWheelUnavailable(
        f"No compatible runtime binary found. Set {env_key} or install the Python package that ships `{binary_name}` for this platform."
    )


def run_subprocess_runtime(
    *,
    binary: str,
    args: list[str],
    timeout_seconds: int,
    classify_error_kind: Callable[[int, str], str | None],
) -> ExecutionResult:
    try:
        result = subprocess.run(
            [binary, *args],
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout_seconds,
            env=sanitized_subprocess_env(),
        )
    except subprocess.TimeoutExpired as exc:
        return ExecutionResult(
            exit_code=1,
            stdout=(exc.stdout or ""),
            stderr=f"runtime command timed out after {timeout_seconds}s",
            error_kind="InternalError",
        )
    except OSError as exc:
        return ExecutionResult(
            exit_code=1,
            stdout="",
            stderr=f"runtime process failed: {exc}",
            error_kind="InternalError",
        )

    return ExecutionResult(
        exit_code=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
        error_kind=classify_error_kind(result.returncode, result.stderr),
    )
