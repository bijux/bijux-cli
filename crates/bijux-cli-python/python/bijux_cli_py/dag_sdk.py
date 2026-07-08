"""Python helpers that delegate to the `bijux-dag` runtime."""

from __future__ import annotations

from collections.abc import Sequence
import json
from pathlib import Path
import sys
from typing import Any

from ._exceptions import InternalError, PlatformWheelUnavailable, ValidationError
from ._runtime import (
    ExecutionResult,
    RuntimeResolution,
    classify_process_error_kind,
    resolve_runtime_binary,
    run_subprocess_runtime,
    runtime_timeout_seconds,
    workspace_runtime_binaries,
)


def _workspace_dag_runtime_binaries() -> list[str]:
    return workspace_runtime_binaries(
        module_file=__file__,
        workspace_crate_dir="bijux-dag-cli",
        binary_name="bijux-dag",
    )


def _resolve_dag_binary() -> RuntimeResolution:
    current_entrypoint = sys.argv[0] if sys.argv and sys.argv[0] else None
    return resolve_runtime_binary(
        binary_name="bijux-dag",
        env_key="BIJUX_DAG_BIN",
        workspace_candidates=_workspace_dag_runtime_binaries(),
        current_entrypoint=current_entrypoint,
    )


def _dag_timeout_seconds() -> int:
    return runtime_timeout_seconds("BIJUX_DAG_PY_SUBPROCESS_TIMEOUT")


def _parse_json_result(result: ExecutionResult) -> dict[str, Any]:
    payload_text = result.stdout.strip() or result.stderr.strip()
    if not payload_text:
        raise InternalError(
            "bijux-dag returned no JSON payload; the runtime command may have failed before emitting structured output"
        )
    try:
        payload = json.loads(payload_text)
    except json.JSONDecodeError as exc:
        raise InternalError(
            f"invalid JSON payload from bijux-dag runtime: {exc}"
        ) from exc
    if not isinstance(payload, dict):
        raise InternalError("bijux-dag JSON payload must be a JSON object")
    return payload


def dag_command_json(argv: Sequence[str]) -> dict[str, Any]:
    runtime = _resolve_dag_binary()
    result = run_subprocess_runtime(
        binary=runtime.binary,
        args=["--json", *argv],
        timeout_seconds=_dag_timeout_seconds(),
        classify_error_kind=classify_process_error_kind,
    )
    return _parse_json_result(result)


def load_dag_graph(graph_path: str | Path) -> dict[str, Any]:
    graph = Path(graph_path).expanduser()
    try:
        payload = json.loads(graph.read_text(encoding="utf-8"))
    except OSError as exc:
        raise InternalError(f"failed to read DAG graph '{graph}': {exc}") from exc
    except json.JSONDecodeError as exc:
        raise ValidationError(f"invalid DAG graph JSON in '{graph}': {exc}") from exc
    if not isinstance(payload, dict):
        raise ValidationError(f"DAG graph '{graph}' must decode to a JSON object")
    return payload


def dag_post_install_diagnostics() -> dict[str, object]:
    warnings_out: list[str] = []
    diagnostics: dict[str, object] = {
        "runtime_binary": None,
        "warnings": warnings_out,
    }
    try:
        diagnostics["runtime_binary"] = _resolve_dag_binary().binary
    except PlatformWheelUnavailable as error:
        warnings_out.append(str(error))
    return diagnostics
